#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use rand::Rng;

use encoding_rs::*;
use lol_html::html_content::ContentType;
use lol_html::{comments, doc_comments, doc_text, element, streaming, text};
use lol_html::{HtmlRewriter, MemorySettings, Settings};

static ASCII_COMPATIBLE_ENCODINGS: [&Encoding; 36] = [
    BIG5,
    EUC_JP,
    EUC_KR,
    GB18030,
    GBK,
    IBM866,
    ISO_8859_2,
    ISO_8859_3,
    ISO_8859_4,
    ISO_8859_5,
    ISO_8859_6,
    ISO_8859_7,
    ISO_8859_8,
    ISO_8859_8_I,
    ISO_8859_10,
    ISO_8859_13,
    ISO_8859_14,
    ISO_8859_15,
    ISO_8859_16,
    KOI8_R,
    KOI8_U,
    MACINTOSH,
    SHIFT_JIS,
    UTF_8,
    WINDOWS_874,
    WINDOWS_1250,
    WINDOWS_1251,
    WINDOWS_1252,
    WINDOWS_1253,
    WINDOWS_1254,
    WINDOWS_1255,
    WINDOWS_1256,
    WINDOWS_1257,
    WINDOWS_1258,
    X_MAC_CYRILLIC,
    X_USER_DEFINED,
];

static SUPPORTED_SELECTORS: [&str; 16] = [
    "*",
    "p",
    "p:not(.firstline)",
    "p.warning",
    "p#myid",
    "p[foo]",
    "p[foo=\"bar\"]",
    "p[foo=\"bar\" i]",
    "p[foo=\"bar\" s]",
    "p[foo~=\"bar\"]",
    "p[foo^=\"bar\"]",
    "p[foo$=\"bar\"]",
    "p[foo*=\"bar\"]",
    "p[foo|=\"bar\"]",
    "p a",
    "p > a",
];

pub fn run_rewriter(data: &[u8]) {
    // fuzzing with randomly picked selector and encoding
    // works much faster (50 times) that iterating over all
    // selectors/encoding per single run. It's recommended
    // to make iterations as fast as possible per fuzzing docs.
    run_rewriter_iter(data, get_random_selector(), get_random_encoding());
}

fn get_random_encoding() -> &'static Encoding {
    let random_encoding_index = rand::thread_rng().gen_range(0..ASCII_COMPATIBLE_ENCODINGS.len());
    ASCII_COMPATIBLE_ENCODINGS[random_encoding_index]
}

fn get_random_selector() -> &'static str {
    let random_selector_index = rand::thread_rng().gen_range(0..SUPPORTED_SELECTORS.len());
    SUPPORTED_SELECTORS[random_selector_index]
}

fn run_rewriter_iter(data: &[u8], selector: &str, encoding: &'static Encoding) {
    let mut rewriter: HtmlRewriter<_> = HtmlRewriter::new(
        Settings {
            enable_esi_tags: true,
            element_content_handlers: vec![
                element!(selector, |el| {
                    el.before(
                        &format!("<!--[ELEMENT('{selector}')]-->"),
                        ContentType::Html,
                    );
                    el.after(
                        &format!("<!--[/ELEMENT('{selector}')]-->"),
                        ContentType::Html,
                    );

                    let replaced = format!("<!--Replaced ({selector}) -->");
                    el.streaming_set_inner_content(streaming!(move |sink| {
                        sink.write_str(&replaced, ContentType::Html);
                        Ok(())
                    }));

                    Ok(())
                }),
                comments!(selector, |c| {
                    c.before(
                        &format!("<!--[COMMENT('{selector}')]-->"),
                        ContentType::Html,
                    );
                    c.after(
                        &format!("<!--[/COMMENT('{selector}')]-->"),
                        ContentType::Html,
                    );

                    Ok(())
                }),
                text!(selector, |t| {
                    t.before(&format!("<!--[TEXT('{selector}')]-->"), ContentType::Html);

                    if t.last_in_text_node() {
                        t.after(&format!("<!--[/TEXT('{selector}')]-->"), ContentType::Html);
                    }

                    Ok(())
                }),
                element!(selector, |el| {
                    el.replace("hey & ya", ContentType::Html);

                    Ok(())
                }),
                element!(selector, |el| {
                    el.remove();

                    Ok(())
                }),
                element!(selector, |el| {
                    el.remove_and_keep_content();

                    Ok(())
                }),
            ],
            document_content_handlers: vec![
                doc_comments!(|c| {
                    c.set_text("123456").unwrap();

                    Ok(())
                }),
                doc_text!(|t| {
                    if t.last_in_text_node() {
                        t.after("BAZ", ContentType::Text);
                    }

                    Ok(())
                }),
            ],
            encoding: encoding.try_into().unwrap(),
            memory_settings: MemorySettings::new(),
            strict: false,
            adjust_charset_on_meta_tag: false,
        },
        |_: &[u8]| {},
    );

    rewriter.write(data).unwrap();
    rewriter.end().unwrap();
}

