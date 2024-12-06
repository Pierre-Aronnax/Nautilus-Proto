// src/mdns/record.rs
use crate::mdns::name::DnsName;
use std::io::Read;
use bytes::Buf;   

#[derive(Debug)]
pub enum DnsRecord {
    A {
        name: DnsName,
        ttl: u32,
        ip: [u8; 4],
    },
    PTR {
        name: DnsName,
        ttl: u32,
        ptr_name: DnsName,
    },
    SRV {
        name: DnsName,
        ttl: u32,
        priority: u16,
        weight: u16,
        port: u16,
        target: DnsName,
    },
    TXT {
        name: DnsName,
        ttl: u32,
        txt_data: Vec<u8>,
    },
    // Add other record types as needed
}

impl DnsRecord {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        match self {
            DnsRecord::A { name, ttl, ip } => {
                name.write(buffer); // Write the name
                buffer.extend_from_slice(&1u16.to_be_bytes()); // TYPE A
                buffer.extend_from_slice(&1u16.to_be_bytes()); // CLASS IN
                buffer.extend_from_slice(&ttl.to_be_bytes());  // TTL
                buffer.extend_from_slice(&4u16.to_be_bytes()); // RDLENGTH
                buffer.extend_from_slice(ip);                 // RDATA (IPv4 address)
            }
            DnsRecord::PTR { name, ttl, ptr_name } => {
                name.write(buffer); // Write the name
                buffer.extend_from_slice(&12u16.to_be_bytes()); // TYPE PTR
                buffer.extend_from_slice(&1u16.to_be_bytes());  // CLASS IN
                buffer.extend_from_slice(&ttl.to_be_bytes());   // TTL
                let mut rdata = Vec::new();
                ptr_name.write(&mut rdata);
                buffer.extend_from_slice(&(rdata.len() as u16).to_be_bytes()); // RDLENGTH
                buffer.extend_from_slice(&rdata);                              // RDATA
            }
            DnsRecord::SRV {
                name,
                ttl,
                priority,
                weight,
                port,
                target,
            } => {
                name.write(buffer); // Write the name
                buffer.extend_from_slice(&33u16.to_be_bytes()); // TYPE SRV
                buffer.extend_from_slice(&1u16.to_be_bytes());  // CLASS IN
                buffer.extend_from_slice(&ttl.to_be_bytes());   // TTL
                let mut rdata = Vec::new();
                rdata.extend_from_slice(&priority.to_be_bytes());
                rdata.extend_from_slice(&weight.to_be_bytes());
                rdata.extend_from_slice(&port.to_be_bytes());
                target.write(&mut rdata);
                buffer.extend_from_slice(&(rdata.len() as u16).to_be_bytes()); // RDLENGTH
                buffer.extend_from_slice(&rdata);                              // RDATA
            }
            DnsRecord::TXT { name, ttl, txt_data } => {
                name.write(buffer); // Write the name
                buffer.extend_from_slice(&16u16.to_be_bytes()); // TYPE TXT
                buffer.extend_from_slice(&1u16.to_be_bytes());  // CLASS IN
                buffer.extend_from_slice(&ttl.to_be_bytes());   // TTL
            
                // Serialize TXT RDATA
                let mut rdata = Vec::new();
                for txt_segment in txt_data.chunks(255) { // Split TXT data into chunks of 255 bytes
                    rdata.push(txt_segment.len() as u8); // Add length byte
                    rdata.extend_from_slice(txt_segment); // Add the actual data
                }
            
                buffer.extend_from_slice(&(rdata.len() as u16).to_be_bytes()); // RDLENGTH
                buffer.extend_from_slice(&rdata);                             // RDATA
            }
        }
    }

    pub fn parse(cursor: &mut std::io::Cursor<&[u8]>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let name = DnsName::parse(cursor)?;
        let rtype = cursor.get_u16();
        #[allow(unused_variables)]
        let rclass = cursor.get_u16();
        let ttl = cursor.get_u32();
        let rdlength = cursor.get_u16();
    
        match rtype {
            1 => { // A Record
                let mut ip = [0u8; 4];
                cursor.read_exact(&mut ip)?;
                Ok(DnsRecord::A { name, ttl, ip })
            }
            12 => { // PTR Record
                let ptr_name = DnsName::parse(cursor)?;
                Ok(DnsRecord::PTR { name, ttl, ptr_name })
            }
            33 => { // SRV Record
                let priority = cursor.get_u16();
                let weight = cursor.get_u16();
                let port = cursor.get_u16();
                let target = DnsName::parse(cursor)?;
                Ok(DnsRecord::SRV { name, ttl, priority, weight, port, target })
            }
            16 => { // TXT Record
                let mut txt_data = vec![0; rdlength as usize];
                cursor.read_exact(&mut txt_data)?;
                Ok(DnsRecord::TXT { name, ttl, txt_data })
            }
            _ => {
                cursor.advance(rdlength as usize);
                Err("Unknown record type".into())
            }
        }
    }
}
