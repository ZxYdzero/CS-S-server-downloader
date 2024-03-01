use std::{fs, io::{self, Write}, path::{Path, PathBuf}};
use async_zip::base::read::seek::ZipFileReader;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};
use tauri::Window;


//查询注册表的函数
pub fn find() -> String {

    let hkcu: RegKey = RegKey::predef(HKEY_LOCAL_MACHINE);
    let place: RegKey = hkcu.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App 240").unwrap();
    let path: String = place.get_value("InstallLocation").unwrap();
    path
}

fn sanitize_file_path(path: &str) -> PathBuf {
    // Replaces backwards slashes
    path.replace('\\', "/")
        // Sanitizes each component
        .split('/')
        .map(sanitize_filename::sanitize)
        .collect()
}

async fn unzip_file(archive: tokio::fs::File, out_dir: &Path) {

    let archive = archive.compat();
    let mut reader = ZipFileReader::new(archive).await.expect("Failed to read zip file");
    for index in 0..reader.file().entries().len() {
        let entry = reader.file().entries().get(index).unwrap();
        let path = out_dir.join(sanitize_file_path(entry.filename().as_str().unwrap()));
        // If the filename of the entry ends with '/', it is treated as a directory.
        // This is implemented by previous versions of this crate and the Python Standard Library.
        // https://docs.rs/async_zip/0.0.8/src/async_zip/read/mod.rs.html#63-65
        // https://github.com/python/cpython/blob/820ef62833bd2d84a141adedd9a05998595d6b6d/Lib/zipfile.py#L528
        let entry_is_dir = entry.dir().unwrap();

        let mut entry_reader = reader.reader_without_entry(index).await.expect("Failed to read ZipEntry");

        if entry_is_dir {
            // The directory may have been created if iteration is out of order.
            if !path.exists() {
                tokio::fs::create_dir_all(&path).await.expect("Failed to create extracted directory");
            }
        } else {
            // Creates parent directories. They may not exist if iteration is out of order
            // or the archive does not contain directory entries.
            let parent = path.parent().expect("A file entry should have parent directories");
            if !parent.is_dir() {
                tokio::fs::create_dir_all(parent).await.expect("Failed to create parent directories");
            }
            let writer = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .await
                .expect("Failed to create extracted file");
            futures_lite::io::copy(&mut entry_reader, &mut writer.compat_write())
                .await
                .expect("Failed to copy to extracted file");

            // Closes the file and manipulates its metadata here if you wish to preserve its metadata from the archive.
        }
    }
}


#[tauri::command]
pub async fn sdownload(window: Window) {

    let mut dest = fs::File::create("./tmp.zip").unwrap();
    let mut downloaded = 0;
    let mut length = 0;

    match reqwest::get("https://gitcode.net/qq_26978213/csgo-server-map/-/archive/qq_26978213-master-patch-98686/csgo-server-map-qq_26978213-master-patch-98686.zip").await {
        Ok(mut response) => {
            if let Some(total) = response.content_length() {
                length = total;
            }
            println!("下载长度 {}", length);
            loop {
                match response.chunk().await {
                    Ok(Some(chunk)) => {
                        downloaded = downloaded + chunk.len() as u64;
                        let percentage = (downloaded as f32 / length as f32) * 100.0;

                        dest.write_all(&chunk).unwrap();
                        window.emit("message", percentage).unwrap();
                    },
                    Ok(None) => {
                        window.emit("message", "下载完成").unwrap();
                        doit(window).await;
                        break;
                    }
                    Err(_) => {
                        window.emit("message", "下载出错").unwrap();
                        break;
                    }
                }
            }
        }
        Err(_) => {}
    }

}



fn copy_dir_recursively(src: &str, dst: &str) -> io::Result<()> {

    // 遍历源目录下的所有目录项
    println!("源目录位于{}", src);
    println!("目标目录位于{}", dst);
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        fs::create_dir_all(&dst).unwrap();


        // 如果是文件，则直接复制到目标目录
        if entry.metadata()?.is_file() {
            let dst_path = format!("{}\\{}", dst, file_name);
            fs::copy(&path, &dst_path)?;
        }
        // 如果是目录，则递归调用复制函数进行复制
        else {

            let dst_path = format!("{}\\{}", dst, file_name);
            copy_dir_recursively(&path.to_string_lossy(), &dst_path).unwrap();
        }
    }

    Ok(())
}

pub fn remove() -> io::Result<()>{
    fs::remove_dir_all("./tmp")?;
    Ok(())
}


pub async fn doit(windows: Window) {
    let mut hecystring = find();
    hecystring.push_str("\\cstrike\\download");
    windows.emit("message",  "正在解压" ).unwrap();
    let archive: tokio::fs::File = tokio::fs::File::open("./tmp.zip").await.expect("Failed to open zip file");
    unzip_file(archive, Path::new("./tmp/")).await;
    windows.emit("message",  "解压完成" ).unwrap();
    let pr = hecystring.clone();
    let mut tryin = String::from("正在导入: ");
    tryin.push_str(&pr);
    windows.emit("message",  &tryin).unwrap();
    copy_dir_recursively("./tmp/csgo-server-map-qq_26978213-master-patch-98686/", &pr.clone()).expect("无法导入地图文件");
    windows.emit("message",  "导入完成" ).unwrap();
    windows.emit("message",  "删除tmp" ).unwrap();
    remove().unwrap();
    windows.emit("message",  "资源文件安装完成" ).unwrap();
}

