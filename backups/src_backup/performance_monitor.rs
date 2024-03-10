use heim::prelude::*;

async fn collect_system_metrics() -> Result<(), heim::Error> {
    let cpu_usage = cpu::usage().await?;
    println!("CPU Usage: {:.2}%", cpu_usage.get::<heim::units::ratio::percent>());

    let memory = memory::memory().await?;
    println!("Memory Usage: {} bytes used", memory.used().get::<heim::units::information::byte>());

    Ok(())
}
