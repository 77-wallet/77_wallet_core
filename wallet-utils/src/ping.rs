use std::net::IpAddr;
use url::Url;

// async fn icmp_ping(host: &str) -> Result<(), Box<dyn std::error::Error>> {
//     let config = Config::default();
//     let mut pinger = Pinger::new(&host.parse()?, config).await?;

//     // 发送 3 个探测包
//     for _ in 0..3 {
//         let (reply, dur) = pinger.ping(0.into(), &[0; 56]).await?;
//         println!("Reply from {}: seq={} time={:.2}ms",
//             reply.source,
//             reply.sequence,
//             dur.as_millis()
//         );
//     }
//     Ok(())
// }

pub async fn ping(host: &str, // 支持域名输入
) -> Result<std::time::Duration, crate::Error> {
    // 解析域名到 IP 地址
    let ips: Vec<IpAddr> = resolve_dns(host).await?;
    let target_ip = ips.first().cloned().ok_or(crate::Error::Icmp(
        crate::error::ping::IcmpError::NoIpAddressesResolved,
    ))?;

    println!("Resolved IP: {}", target_ip);
    // 创建 Pinger 实例
    let (_packet, duration) = surge_ping::ping(target_ip, &[1, 2, 3, 4, 5, 6, 7, 8])
        .await
        .map_err(|e| crate::Error::Icmp(e.into()))?;
    //  {
    //     Ok((_packet, duration)) => println!("_packet: {_packet:#?}, duration: {:.2?}", duration),
    //     Err(e) => println!("{:?}", e),
    // };
    Ok(duration)
}

async fn resolve_dns(input: &str) -> Result<Vec<IpAddr>, crate::Error> {
    let host = if input.starts_with("http://") || input.starts_with("https://") {
        // 解析 URL 获取主机名
        let parsed_url = Url::parse(input).map_err(crate::ParseError::Uri)?;
        parsed_url
            .host_str()
            .ok_or(crate::Error::Icmp(
                crate::error::ping::IcmpError::InvalidURL,
            ))?
            .to_string()
    } else {
        // 直接使用裸域名
        input.to_string()
    };

    println!("Resolving DNS for host: {}", host);
    // 使用 Tokio 的异步 DNS 解析
    let addrs = tokio::net::lookup_host(format!("{}:0", host)).await?;

    // 提取 IP 地址并优先 IPv4
    let mut ips: Vec<IpAddr> = addrs.map(|sa| sa.ip()).collect();
    ips.sort_by(|a, b| a.is_ipv4().cmp(&b.is_ipv4()).reverse());

    Ok(ips)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping() -> Result<(), Box<dyn std::error::Error>> {
        // let host = "www.baidu.com";
        let host = "apprpc.safew.cc";
        // let host = "https://apprpc.safew.cc/eth";
        // let host = "rpc.88ai.fun";
        ping(host).await?;
        Ok(())
    }
}
