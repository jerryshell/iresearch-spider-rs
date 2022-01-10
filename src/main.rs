use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long, default_value_t = 64)]
    parallel_requests: usize,

    #[clap(short = 'b', long, default_value_t = 0)]
    id_range_begin: usize,

    #[clap(short = 'e', long, default_value_t = 5000)]
    id_range_end: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let research_report_list_arc = iresearch_spider_rs::fech_research_report_list_by_id_range(
        (args.id_range_begin, args.id_range_end),
        args.parallel_requests,
    )
    .await?;

    iresearch_spider_rs::write_to_csv(research_report_list_arc).await?;

    Ok(())
}
