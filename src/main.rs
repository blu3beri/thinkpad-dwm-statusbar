use std::path::Path;
use std::fs::File;
use std::io::Read; 
use std::process::Command; 
use std::time::Duration;
use chrono::offset::Local;

#[link(name = "X11")]
extern "C" {
    fn XOpenDisplay(screen: usize) -> usize;
    fn XDefaultRootWindow(display: usize) -> usize;
    fn XStoreName(display: usize, window: usize, name: *const u8) -> i32;
    fn XFlush(display: usize) -> i32;
}

fn read(p: impl AsRef<Path>) -> std::io::Result<String> {
    let mut file = File::open(p)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn batteries() -> std::io::Result<(String, String)> {

    let bat_0 = battery(0)?;
    let bat_1 = battery(1)?;

    Ok((bat_0, bat_1))
}

fn battery(id: u8) -> std::io::Result<String> {
    let bat_status = read(format!("/sys/class/power_supply/BAT{id}/status"))?;

    if bat_status == "Charging" {
        return Ok(String::from("C"))
    }
    
    let cap = read(format!("/sys/class/power_supply/BAT{id}/capacity"))?;

    Ok(format!("{cap}%"))
}

fn sound() -> std::io::Result<String> {
    let status = Command::new("amixer")
        .arg("get")
        .arg("Master")
        .output()?
        .stdout;

    let stdout = std::str::from_utf8(&status).unwrap();

    let mono = stdout.split("\n").collect::<Vec<&str>>()[4];
    let words = mono.split_whitespace().collect::<Vec<&str>>();
    let volume = words[3].replace("[","").replace("]", "");
    let audible = words[5].replace("[", "").replace("]", "");

    if audible == "off" {
        return Ok(String::from("MUTED"));
    }

    Ok(volume)
}

fn backlight() -> std::io::Result<String> {
    let status = Command::new("xbacklight")
        .output()?
        .stdout;
    let stdout = std::str::from_utf8(&status).unwrap();

    let percentage = stdout.split(".").next().unwrap().to_string();

    Ok(format!("{percentage}%"))
}

fn main() -> std::io::Result<()> {
    let display = unsafe { XOpenDisplay(0) };
    let window = unsafe { XDefaultRootWindow(display) };

    loop {
        let time = Local::now().format("%Y-%m-%d %H:%M:%S");
        let (bat0, bat1) = batteries()?;
        let vol = sound()?;
        let brightness = backlight()?;
        let name = format!("[V: {vol}] [B: {brightness}] [0: {bat0} 1: {bat1}] [{time}]\0").replace("\n", "");

        unsafe { XStoreName(display, window, name.as_ptr()) };
        unsafe { XFlush(display) };

        std::thread::sleep(Duration::from_secs(1));
    }
}
