// /packet.rs
use crate::mdns::record::DnsRecord;
use crate::mdns::name::DnsName;

use bytes::Buf;

#[derive(Debug)]
pub struct DnsPacket {
    pub id: u16,
    pub flags: u16,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub authorities: Vec<DnsRecord>,
    pub additionals: Vec<DnsRecord>,
}

impl DnsPacket {
    pub fn new() -> Self {
        DnsPacket {
            id: 0,
            flags: 0x8400, // Standard Query Response, Authoritative Answer
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            additionals: Vec::new(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // Serialize header
        buffer.extend_from_slice(&self.id.to_be_bytes());
        buffer.extend_from_slice(&self.flags.to_be_bytes());
        buffer.extend_from_slice(&(self.questions.len() as u16).to_be_bytes());
        buffer.extend_from_slice(&(self.answers.len() as u16).to_be_bytes());
        buffer.extend_from_slice(&(self.authorities.len() as u16).to_be_bytes());
        buffer.extend_from_slice(&(self.additionals.len() as u16).to_be_bytes());

        // Serialize questions (if needed)

        // Serialize records
        for record in &self.answers {
            record.write(&mut buffer);
        }
        for record in &self.authorities {
            record.write(&mut buffer);
        }
        for record in &self.additionals {
            record.write(&mut buffer);
        }

        buffer
    }

    pub fn parse(data: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut cursor = std::io::Cursor::new(data);
    
        // Parse the header
        let id = cursor.get_u16();
        let flags = cursor.get_u16();
        let qdcount = cursor.get_u16();
        let ancount = cursor.get_u16();
        let nscount = cursor.get_u16();
        let arcount = cursor.get_u16();
    
        // Parse questions
        let mut questions = Vec::new();
        for _ in 0..qdcount {
            questions.push(DnsQuestion::parse(&mut cursor)?);
        }
    
        // Parse answers
        let mut answers = Vec::new();
        for _ in 0..ancount {
            answers.push(DnsRecord::parse(&mut cursor)?);
        }
    
        // Parse authorities (optional)
        let mut authorities = Vec::new();
        for _ in 0..nscount {
            authorities.push(DnsRecord::parse(&mut cursor)?);
        }
    
        // Parse additional records (optional)
        let mut additionals = Vec::new();
        for _ in 0..arcount {
            additionals.push(DnsRecord::parse(&mut cursor)?);
        }
    
        Ok(DnsPacket {
            id,
            flags,
            questions,
            answers,
            authorities,
            additionals,
        })
    }
}

// Define DnsQuestion if needed
#[allow(dead_code)]
#[derive(Debug)]
pub struct DnsQuestion {
    pub qname: DnsName,
    pub qtype: u16,
    pub qclass: u16,
}

impl DnsQuestion {
    pub fn parse(cursor: &mut std::io::Cursor<&[u8]>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let qname = DnsName::parse(cursor)?;
        let qtype = cursor.get_u16();
        let qclass = cursor.get_u16();
        Ok(DnsQuestion { qname, qtype, qclass })
    }
}