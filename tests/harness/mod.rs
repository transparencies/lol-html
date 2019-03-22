use encoding_rs::*;

#[macro_use]
pub mod functional_testing;

#[macro_use]
pub mod parsing;

mod unescape;

pub static ASCII_COMPATIBLE_ENCODINGS: [&Encoding; 36] = [
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

pub struct TestOutput {
    bytes: Vec<u8>,
    encoding: &'static Encoding,
    finalizing_chunk_received: bool,
}

impl TestOutput {
    pub fn new(encoding: &'static Encoding) -> Self {
        TestOutput {
            bytes: Vec::default(),
            encoding,
            finalizing_chunk_received: false,
        }
    }

    pub fn push(&mut self, chunk: &[u8]) {
        if chunk.is_empty() {
            self.finalizing_chunk_received = true;
        } else {
            assert!(
                !self.finalizing_chunk_received,
                "Chunk written to the output after the finalizing chunk."
            );

            self.bytes.extend_from_slice(chunk);
        }
    }
}

impl Into<String> for TestOutput {
    fn into(self) -> String {
        assert!(
            self.finalizing_chunk_received,
            "Finalizing chunk for the output hasn't been received."
        );

        self.encoding
            .decode_without_bom_handling(&self.bytes)
            .0
            .into_owned()
    }
}

macro_rules! create_test {
    ($name:expr, $body:tt) => {{
        use test::{ShouldPanic, TestDesc, TestDescAndFn, TestFn, TestName};

        TestDescAndFn {
            desc: TestDesc {
                name: TestName::DynTestName($name),
                ignore: false,
                should_panic: ShouldPanic::No,
                allow_fail: false,
            },
            testfn: TestFn::DynTestFn(Box::new(move || $body)),
        }
    }};
}

macro_rules! test_fixture {
    ($fixture_name:expr, { $(test($name:expr, $body:tt);)+}) => {
        use test::TestDescAndFn;
        use std::fmt::Write;

        pub fn get_tests() -> Vec<TestDescAndFn> {
            let mut tests = Vec::default();

            $({
                let mut name = String::new();

                write!(&mut name, "{} - {}", $fixture_name, $name).unwrap();

                tests.push(create_test!(name, $body));
            })+

            tests
        }
    };
}

macro_rules! test_modules {
    ($($m:ident),+) => {
        $(mod $m;)+

        use test::TestDescAndFn;

        pub fn get_tests() -> Vec<TestDescAndFn> {
            let mut tests = Vec::default();

            $(tests.extend($m::get_tests());)+

            tests
        }
    };
}
