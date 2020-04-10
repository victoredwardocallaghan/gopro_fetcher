// Copyright (C) 2020, Edward O'Callaghan.
//
// This program is free software; you can redistribute it and/or
// modify it under the terms of the GNU General Public License
// as published by the Free Software Foundation; either version 2
// of the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301, USA.

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate reqwest;

use std::io::copy;
use std::fs::File;
use std::path::Path;
use reqwest::{Error};
use wake_on_lan;

#[derive(Deserialize, Debug)]
struct InfoObj {
    info: Info,
}

#[derive(Deserialize, Debug)]
struct Info {
    model_number: u32,
    model_name: String,
    firmware_version: String,
    serial_number: String,
    board_type: String,
    ap_mac: String,
    ap_ssid: String,
    ap_has_default_credentials: String,
    capabilities: String,
    lens_count: String,
    update_required: String,
}

fn format_req(s: &str) -> String {
    format!("http://10.5.5.9:8080/gp/{param}",
            param = s)
}

fn get_info() -> Result<(), Error> {
    let req_url = format_req("gpControl/info");
    let obj = reqwest::blocking::get(&req_url)?.json::<InfoObj>()?;
    println!("{:#?}", obj.info);
    Ok(())
}

#[derive(Deserialize, Debug)]
struct MediaObj {
    id: String,
    media: Vec<Media>,
}

#[derive(Deserialize, Debug)]
struct Media {
    d: String,
    fs: Vec<MediaFile>,
}

#[derive(Deserialize, Debug)]
struct MediaFile {
    n: String,
    cre: String,
    r#mod: String,
    ls: String,
    s: String,
}

fn get_media_list() -> Result<MediaObj, Error> {
    let req_url = format_req("gpMediaList");
    let obj = reqwest::blocking::get(&req_url)?.json::<MediaObj>()?;
    //println!("{:#?}", obj.media);
    Ok(obj)
}

fn fetch_media_file(dl_root: &Path, path: &str, file: &str)
    -> Result<(), std::io::Error> {
    let req_url = format!("http://10.5.5.9:8080/videos/DCIM/{d}/{f}",
                            d = path, f = file);
    let mut resp = reqwest::blocking::get(&req_url).unwrap();

    let mut dest = {
        let fname = resp
            .url()
            .path_segments()
            .and_then(|segment| segment.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        println!("file to download: '{}'", fname);
        let fname = dl_root.join(fname);
        println!("will be located under: '{:?}'", fname);
        File::create(fname)?
    };
    copy(&mut resp, &mut dest)?;
    Ok(())
}

fn delete_media_file(path: &str, file: &str) -> Result<(), Error> {
    let s = format!("gpControl/command/storage/delete?p={p}/{f}", p=path, f=file);
    let req_url = format_req(&s);
    //reqwest::blocking::get(&req_url)?;
    println!("DEBUG:<pretend for now>: {:?}", req_url);
    Ok(())
}

fn enumate_files_to_dl() -> Result<(), Error> {
    let save_dir = Path::new("gp_dl");
    let media_obj = get_media_list()?;
    for m in media_obj.media {
        let root_d = m.d;
        println!("root directory path = {:?}", root_d);
        for mf in m.fs {
            let file_name = mf.n;
            if fetch_media_file(&save_dir, &root_d, &file_name).is_ok() {
                delete_media_file(&root_d, &file_name)?;
            }
        }
    }
    Ok(())
}

fn wake_gopro() -> Result<(), std::io::Error> {
    let mac_addr: [u8; 6] = [0xZZ, 0xZZ, 0xZZ, 0xZZ, 0xZZ, 0xZZ];
    let magic_packet = wake_on_lan::MagicPacket::new(&mac_addr);
    magic_packet.send()
}

fn main() {
    println!("Fetching Go$hit media content off SDCard..!");
    if wake_gopro().is_err() {
        println!("failed to WoL");
    }
    if get_info().is_err() {
        println!("failed to get gopro info");
    }
    if enumate_files_to_dl().is_err() {
        println!("failed to enumerate dl files");
    }
}
