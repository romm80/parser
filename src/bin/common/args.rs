use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
pub struct Convert {
    #[clap(long)]
    pub input: Option<String>,
    #[clap(long)]
    pub input_format: Format,
    #[clap(long)]
    pub output: Option<String>,
    #[clap(long)]
    pub output_format: Format,
}

#[derive(Debug, Parser)]
pub struct Compare {
    #[clap(long)]
    pub file1: String,
    #[clap(long)]
    pub file1_format: Format,
    #[clap(long)]
    pub file2: String,
    #[clap(long)]
    pub file2_format: Format,
}

#[derive(Debug, Parser, Clone, ValueEnum)]
pub enum Format {
    Csv,
    Bin,
    Txt,
}

impl Into<parser::Format> for Format {
    fn into(self) -> parser::Format {
        match self {
            Format::Csv => {parser::Format::Csv}
            Format::Bin => {parser::Format::Bin}
            Format::Txt => {parser::Format::Txt}
        }
    }
}