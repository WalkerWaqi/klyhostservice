use curl::easy::Easy;

pub fn download() {
    let mut dst = Vec::new();
    let mut easy = Easy::new();
    easy.url("http://172.27.128.202:28080/FreeRdp-wfreerdp.7z")
        .unwrap();

    let mut transfer = easy.transfer();
    transfer
        .write_function(|data| {
            dst.extend_from_slice(data);
            println!("{} {}", data.len(), dst.len());
            Ok(data.len())
        })
        .unwrap();
    transfer.perform().unwrap();
}
