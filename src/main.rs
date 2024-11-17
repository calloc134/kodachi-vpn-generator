use log::info;
use md5;
use rand::{distributions::Alphanumeric, Rng};
use reqwest;
use std::{
    fs::File,
    io::{Read, Write},
};
use zip::ZipArchive;

fn main() {
    // ロガーの初期化
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let vpn_zip = "https://digi77.com/software/kodachi/kodachi-vpn.zip";

    // ランダムな文字列を生成
    let f1: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    // md5ハッシュ化
    let digest = md5::compute(&f1);
    let e = format!("{:x}", digest);
    let auth_user_username_output = format!("kodachi|{}", e);
    let auth_user_password_output = format!("cf90b117a31e7c2bb53cac3186b867b0");
    let auth_user_output = format!(
        "{}\n{}",
        auth_user_username_output, auth_user_password_output
    );
    info!("ユーザ名を生成しました。: {}", auth_user_username_output);

    let password = "a30@06e61-79-34-88-A4-C3@".as_bytes();

    info!("VPN設定ファイルをダウンロードします。");
    // ZIPファイルをダウンロード
    let resp = reqwest::blocking::get(vpn_zip).unwrap();
    let mut archive = ZipArchive::new(std::io::Cursor::new(resp.bytes().unwrap())).unwrap();
    info!("VPN設定ファイルを解凍します。");

    // ZIPファイル内のファイルを解凍
    for i in 0..archive.len() {
        let mut file = archive.by_index_decrypt(i, password).unwrap();
        let outpath = file.mangled_name();

        // ファイル名が kodachi-vpn.ovpn の場合のみ保存
        if file.name() == "kodachi-vpn.ovpn" {
            // ファイルの中身はopenvpnの設定ファイル

            // ファイルの中身を読み込む
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            // ファイル内のremoteよりIPアドレスを取得
            let remote = contents
                .lines()
                .find(|line| line.starts_with("remote"))
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap();
            info!("VPNサーバのIPアドレス: {}", remote);

            // auth-user-pass auth という行があるので、その行を書き換え、ハードコードした認証情報を使うようにする
            info!("VPN設定ファイルを書き換えます。");
            let auth_user_pass =
                format!("<auth-user-pass>\n{}\n</auth-user-pass>", auth_user_output);
            let new_contents =
                contents.replace("auth-user-pass /etc/openvpn/auth", &auth_user_pass);

            // ファイルに書き込み
            let mut outfile = File::create(&outpath).unwrap();
            outfile.write_all(new_contents.as_bytes()).unwrap();
            info!("VPN設定ファイルを保存しました。");
        }
    }
}
