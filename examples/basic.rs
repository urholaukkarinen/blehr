use futures::StreamExt;

use blehr::{Error, Scanner};

#[tokio::main]
async fn main() -> Result<(), Box<Error>> {
    let mut scanner = Scanner::new().await?;

    loop {
        scanner.start().await?;

        println!("Scanning for HR sensors ...");
        let mut sensor = scanner.next_sensor().await?.unwrap();

        println!(
            "Found {}",
            sensor
                .name()
                .await
                .unwrap_or_else(|| "unknown sensor".to_string())
        );

        scanner.stop().await?;

        if let Ok(mut hr_stream) = sensor.hr_stream().await {
            while let Some(hr) = hr_stream.next().await {
                println!("hr: {:?}", hr);
            }
        }
    }
}
