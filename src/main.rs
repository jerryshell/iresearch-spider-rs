use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long, default_value_t = 64)]
    parallel_requests: usize,

    #[clap(short = 'b', long, default_value_t = 0)]
    report_id_range_begin: i64,

    #[clap(short = 'e', long, default_value_t = 5000)]
    report_id_range_end: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let parallel_requests = args.parallel_requests;
    let report_id_range_begin = args.report_id_range_begin;
    let report_id_range_end = args.report_id_range_end;

    let report_id_list = (report_id_range_begin..report_id_range_end).collect::<Vec<i64>>();

    let research_report_list_arc = iresearch_spider_rs::fech_research_report_list_by_id_list(
        reqwest::Client::new(),
        report_id_list,
        parallel_requests,
    )
    .await?;

    iresearch_spider_rs::write_to_csv(research_report_list_arc).await?;

    Ok(())
}
