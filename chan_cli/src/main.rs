use chan_core::analyzer::analyzer::Analyzer;
use chan_core::kline::kline_unit::KLineUnit;
use chrono::NaiveDateTime;
use csv::Reader;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
struct CsvRecord {
    timestamp: NaiveDateTime,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let data_dir = Path::new("/opt/data/raw_data");

    // 遍历目录下的所有csv文件
    for entry in std::fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("csv") {
            println!("Processing file: {:?}", path);
            process_csv_file(&path)?;
        }
    }

    Ok(())
}

fn process_csv_file(path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = Reader::from_reader(file);
    let mut kline_units = Vec::new();

    for result in rdr.records() {
        let record = result?;

        // Parse CSV record
        let csv_record = parse_csv_record(&record)?;

        // Convert to KLineUnit
        let klu = KLineUnit::new(
            csv_record.timestamp.timestamp() as usize,
            csv_record.open,
            csv_record.high,
            csv_record.low,
            csv_record.close,
            csv_record.volume,
        );

        kline_units.push(klu);
    }

    // Sort by timestamp
    kline_units.sort_by_key(|k| k.timestamp);

    // Create and run analyzer
    let mut analyzer = Analyzer::new();
    analyzer.update(&kline_units)?;

    // Print analysis results
    println!("Analysis completed for {:?}", path);
    println!("Number of K-line units: {}", kline_units.len());
    println!(
        "First timestamp: {}",
        NaiveDateTime::from_timestamp_opt(kline_units.first().unwrap().timestamp as i64, 0)
            .unwrap()
    );
    println!(
        "Last timestamp: {}",
        NaiveDateTime::from_timestamp_opt(kline_units.last().unwrap().timestamp as i64, 0).unwrap()
    );

    Ok(())
}

fn parse_csv_record(record: &csv::StringRecord) -> Result<CsvRecord, Box<dyn Error>> {
    let timestamp = NaiveDateTime::parse_from_str(&record[0], "%Y-%m-%d %H:%M:%S")?;

    Ok(CsvRecord {
        timestamp,
        open: record[1].parse()?,
        high: record[2].parse()?,
        low: record[3].parse()?,
        close: record[4].parse()?,
        volume: record[5].parse()?,
    })
}
