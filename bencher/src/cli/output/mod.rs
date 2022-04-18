use crate::cli::clap::CliOutput;

const BENCHER_URL: &str = "https://bencher.dev";

#[derive(Debug, Default)]
pub enum Output {
    Headless,
    #[default]
    Web,
    Url(String),
}

impl From<CliOutput> for Output {
    fn from(output: CliOutput) -> Self {
        if output.headless {
            return Self::Headless;
        }
        if output.web {
            return Self::Web;
        }
        if let Some(url) = output.url {
            return Self::Url(url);
        }
        Self::Web
    }
}

impl Output {
    pub fn open(&self, report_str: &str) -> Result<(), std::io::Error> {
        match &self {
            Self::Headless => Ok(println!("{report_str}")),
            Self::Web => open_url(BENCHER_URL, report_str),
            Self::Url(url) => open_url(url, report_str),
        }
    }
}

fn open_url(url: &str, report_str: &str) -> Result<(), std::io::Error> {
    open::that(url)
}
