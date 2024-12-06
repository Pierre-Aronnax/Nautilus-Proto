// src/name.rs
use std::io::Read;
use bytes::Buf;   
use std::fmt;

#[derive(Clone,PartialEq,Debug)]
pub struct DnsName {
  labels: Vec<String>,
}


impl DnsName {
  pub fn new(name: &str) -> Result<Self, String> {
      let labels: Vec<String> = name
          .split('.')
          .filter(|label| !label.is_empty())
          .map(|label| label.to_string())
          .collect();

      // Validate label lengths
      for label in &labels {
          if label.len() > 63 {
              return Err(format!("Label '{}' exceeds 63 characters", label));
          }
      }

      Ok(DnsName { labels })
  }

  pub fn write(&self, buffer: &mut Vec<u8>) {
      for label in &self.labels {
          buffer.push(label.len() as u8);
          buffer.extend_from_slice(label.as_bytes());
      }
      buffer.push(0x00); // End of the domain name
  }

  pub fn parse(cursor: &mut std::io::Cursor<&[u8]>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
    let mut labels = Vec::new();
    loop {
        let len = cursor.get_u8();
        if len == 0 {
            break;
        }
        let mut label = vec![0; len as usize];
        cursor.read_exact(&mut label)?;
        labels.push(String::from_utf8(label)?);
    }
    Ok(DnsName { labels })
}
}


impl fmt::Display for DnsName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.labels.join("."))
    }
}