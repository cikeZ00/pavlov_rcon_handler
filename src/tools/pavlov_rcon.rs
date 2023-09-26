use std::error::Error;
use std::io::{Read, Write, BufReader};
use std::net::TcpStream;
use std::time::Duration;
use std::thread;
use serde_json::Value;
use md5;

pub struct PavlovRCON {
    ip: String,
    port: u16,
    password: String,
    timeout: Duration,
}

impl PavlovRCON {
    pub(crate) fn new(ip: &str, port: u16, password: &str, timeout: u64) -> Self {
        PavlovRCON {
            ip: ip.to_string(),
            port,
            password: format!("{:x}", md5::compute(password.as_bytes())),
            timeout: Duration::from_secs(timeout),
        }
    }

    pub(crate) fn multisend(&self, commands: &[&str]) -> Result<Vec<Value>, Box<dyn Error>> {
        let mut stream = TcpStream::connect(format!("{}:{}", self.ip, self.port))?;
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;
        let time_to_sleep = Duration::from_millis(100);

        self._auth(&mut stream)?;

        let mut responses = Vec::new();

        for command in commands {
            self._send(&mut stream, command)?;
            let response = self._recv(&mut stream)?;
            responses.push(response);
            thread::sleep(time_to_sleep);
        }

        Ok(responses)
    }
    pub(crate) fn send(&self, command: &str) -> Result<Value, Box<dyn Error>> {
        let mut stream = TcpStream::connect(format!("{}:{}", self.ip, self.port))?;
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;

        self._auth(&mut stream)?;

        self._send(&mut stream, command)?;
        let response = self._recv(&mut stream)?;

        Ok(response)
    }


    fn _auth(&self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        self._send(stream, &self.password)?;
        let time_to_sleep = Duration::from_millis(160);
        thread::sleep(time_to_sleep);

        let response = self._recv(stream)?;

        if !response["data"].as_str().map(|s| s.contains("Authenticated=1")).unwrap_or(false) {
            return Err("Authentication failed".into());
        }

        Ok(())
    }

    pub(crate) fn _send(&self, stream: &mut TcpStream, data: &str) -> Result<(), Box<dyn Error>> {
        stream.write_all(data.as_bytes())?;
        Ok(())
    }

    fn _disconnect(&self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        self._send(stream, "Disconnect")?;
        Ok(())
    }

    fn _recv(&self, stream: &mut TcpStream) -> Result<Value, Box<dyn Error>> {
        let mut reader = BufReader::new(stream);
        let mut full_msg = [0; 2048];
        reader.read(&mut full_msg)?;

        let converted = std::str::from_utf8(&full_msg)?;
        let cleaned_data = converted.chars().filter(|&c| c.is_ascii() && !c.is_ascii_control()).collect::<String>();

        // Try parsing the input JSON
        match serde_json::from_str(&cleaned_data) {
            Ok(json_data) => Ok(json_data), // If valid JSON, return it
            Err(_) => {
                // If invalid JSON, create a generic JSON object
                let generic_json = serde_json::json!({
                "Notice": "Converted to JSON",
                "data": cleaned_data,
            });

                Ok(generic_json)
            }
        }
    }
}