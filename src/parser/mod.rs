#[macro_use]
mod state_machine;

mod lexer;
mod tag_scanner;
mod tree_builder_simulator;

use self::lexer::Lexer;
pub(crate) use self::lexer::{
    AttributeBuffer, AttributeOutline, Lexeme, LexemeSink, NonTagContentLexeme,
    NonTagContentTokenOutline, TagLexeme, TagTokenOutline,
};
use self::state_machine::StateMachine;
pub(crate) use self::state_machine::{ActionError, ActionResult};
pub(crate) use self::tag_scanner::TagHintSink;
use self::tag_scanner::TagScanner;
pub use self::tree_builder_simulator::ParsingAmbiguityError;
use self::tree_builder_simulator::{TreeBuilderFeedback, TreeBuilderSimulator};
use crate::rewriter::RewritingError;

// NOTE: tag scanner can implicitly force parser to switch to
// the lexer mode if it fails to get tree builder feedback. It's up
// to consumer to switch the parser back to the tag scan mode in
// the tag handler.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ParserDirective {
    WherePossibleScanForTagsOnly,
    Lex,
}

pub(crate) struct ParserContext<S> {
    output_sink: S,
    tree_builder_simulator: TreeBuilderSimulator,
    /// Amount of bytes consumed by previous calls to `parse()`,
    /// i.e. number of bytes from the start of the document until the start of the current input slice
    previously_consumed_byte_count: usize,
}

pub(crate) trait ParserOutputSink: LexemeSink + TagHintSink {}

// Pub only for integration tests
pub struct Parser<S> {
    lexer: Lexer<S>,
    tag_scanner: TagScanner<S>,
    current_directive: ParserDirective,
    context: ParserContext<S>,
}

// public only for integration tests
#[allow(private_bounds, private_interfaces)]
impl<S: ParserOutputSink> Parser<S> {
    #[inline]
    #[must_use]
    pub fn new(output_sink: S, initial_directive: ParserDirective, strict: bool) -> Self {
        let context = ParserContext {
            output_sink,
            previously_consumed_byte_count: 0,
            tree_builder_simulator: TreeBuilderSimulator::new(strict),
        };

        Self {
            lexer: Lexer::new(),
            tag_scanner: TagScanner::new(),
            current_directive: initial_directive,
            context,
        }
    }

    // generic methods tend to be inlined, but this one is called from a couple of places,
    // and has cheap-to-pass non-constants args, so it won't benefit from being merged into its callers.
    // It's better to outline it, and let its callers be inlined.
    #[inline(never)]
    pub fn parse(&mut self, input: &[u8], last: bool) -> Result<usize, RewritingError> {
        let mut parse_result = match self.current_directive {
            ParserDirective::WherePossibleScanForTagsOnly => {
                self.tag_scanner
                    .run_parsing_loop(&mut self.context, input, last)
            }
            ParserDirective::Lex => self.lexer.run_parsing_loop(&mut self.context, input, last),
        };

        loop {
            let unboxed = match parse_result {
                Ok(unreachable) => match unreachable {},
                Err(boxed) => *boxed,
            };
            match unboxed {
                ActionError::EndOfInput {
                    consumed_byte_count,
                } => {
                    self.context.previously_consumed_byte_count += consumed_byte_count;
                    return Ok(consumed_byte_count);
                }
                ActionError::ParserDirectiveChangeRequired(new_directive, sm_bookmark) => {
                    self.current_directive = new_directive;

                    trace!(@continue_from_bookmark sm_bookmark, self.current_directive, input);

                    parse_result = match self.current_directive {
                        ParserDirective::WherePossibleScanForTagsOnly => self
                            .tag_scanner
                            .continue_from_bookmark(&mut self.context, input, last, sm_bookmark),
                        ParserDirective::Lex => self.lexer.continue_from_bookmark(
                            &mut self.context,
                            input,
                            last,
                            sm_bookmark,
                        ),
                    };
                }
                ActionError::RewritingError(err) => return Err(err),
                ActionError::Internal(err) => {
                    return Err(RewritingError::ContentHandlerError(err.into()))
                }
            }
        }
    }

    pub fn get_dispatcher(&mut self) -> &mut S {
        &mut self.context.output_sink
    }
}
