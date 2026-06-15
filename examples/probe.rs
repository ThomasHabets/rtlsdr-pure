use futures_lite::future::block_on;
use rtlsdr_pure::{GainMode, Result, open_first};

fn main() {
    if let Err(err) = block_on(run()) {
        eprintln!("probe failed: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let frequency_hz = parse_arg(args.next(), 100_000_000);
    let sample_rate_hz = parse_arg(args.next(), rtlsdr_pure::DEFAULT_SAMPLE_RATE);
    let read_len = parse_arg(args.next(), 16 * 32 * 512usize);

    let mut sdr = open_first().await?;
    println!(
        "opened {:04x}:{:04x} {}",
        sdr.vendor_id(),
        sdr.product_id(),
        sdr.known_name().unwrap_or("RTL-SDR")
    );
    if let Some(manufacturer) = sdr.manufacturer() {
        println!("manufacturer: {manufacturer}");
    }
    if let Some(product) = sdr.product() {
        println!("product: {product}");
    }
    println!("tuner: {:?}", sdr.tuner_kind());

    let actual_rate = sdr.set_sample_rate(sample_rate_hz).await?;
    println!("sample rate: {actual_rate} Hz");

    if sdr.tuner_kind().is_supported() {
        sdr.set_tuner_gain(GainMode::Auto).await?;
        sdr.set_center_frequency(frequency_hz).await?;
        println!("center frequency: {frequency_hz} Hz");
    } else {
        println!("center frequency: skipped for unsupported tuner");
    }

    sdr.reset_buffer().await?;
    let bytes = sdr.read_bytes(read_len).await?;
    let pair_count = bytes.len() / 2;
    let (mean_i, mean_q) = if pair_count == 0 {
        (0.0, 0.0)
    } else {
        let (sum_i, sum_q) = bytes
            .chunks_exact(2)
            .fold((0u64, 0u64), |(sum_i, sum_q), pair| {
                (sum_i + pair[0] as u64, sum_q + pair[1] as u64)
            });
        (
            sum_i as f64 / pair_count as f64,
            sum_q as f64 / pair_count as f64,
        )
    };

    println!(
        "read {} bytes ({} I/Q pairs), mean I={mean_i:.2}, mean Q={mean_q:.2}",
        bytes.len(),
        pair_count
    );
    println!("first 16 bytes: {:02x?}", &bytes[..bytes.len().min(16)]);

    Ok(())
}

fn parse_arg<T>(arg: Option<String>, default: T) -> T
where
    T: std::str::FromStr,
{
    arg.and_then(|value| value.parse().ok()).unwrap_or(default)
}
