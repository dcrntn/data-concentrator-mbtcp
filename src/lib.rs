use tokio_modbus::prelude::*;

fn build_dest(ip: String, port: String) -> String {
    let mut dest = ip.to_owned();
    dest.push_str(":");
    dest.push_str(&port);
    dest
}

pub async fn read_mb(
    ip: String,
    port: String,
    reg: u16,
    size: u16,
) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
    let dest = build_dest(ip, port);
    let socket_addr = dest.parse().unwrap();

    let mut ctx = tcp::connect(socket_addr).await?;

    let data = ctx.read_input_registers(reg, size).await?;

    Ok(data)
}

pub async fn write_mb(
    ip: String,
    port: String,
    reg: u16,
    to_write: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let dest = build_dest(ip, port);
    let socket_addr = dest.parse().unwrap();
    let mut ctx = tcp::connect(socket_addr).await?;

    let parsed_to_write: u16 = to_write.parse().unwrap();

    ctx.write_single_register(reg, parsed_to_write).await?;

    Ok(())
}
