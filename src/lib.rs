use crate::AccountRelation::{Direct, Reseller};
use std::fmt::Formatter;

pub type Result<T> = ::std::result::Result<T, Box<AdsTxtError>>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AdsTxtError {
    message: String,
}

impl AdsTxtError {
    pub fn new(message: &str) -> AdsTxtError {
        AdsTxtError {
            message: message.to_string(),
        }
    }
}

fn ads_txt_error<T>(message: &str) -> Result<T> {
    Err(Box::new(AdsTxtError::new(message)))
}

impl std::fmt::Display for AdsTxtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct AdsTxt {
    pub records: Vec<DataRecord>,
    pub variables: Vec<Variable>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Variable {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AccountRelation {
    Direct,
    Reseller,
}

impl AccountRelation {
    fn parse(text: &str) -> Result<AccountRelation> {
        let relation = text.trim().to_lowercase();

        if &relation == "direct" {
            Ok(Direct)
        } else if &relation == "reseller" {
            Ok(Reseller)
        } else {
            ads_txt_error(&format!("Invalid account relation: {}", text))
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DataRecord {
    pub domain: String,
    pub publisher_id: String,
    pub acc_relation: AccountRelation,
    pub cert_authority: Option<String>,
}

impl DataRecord {
    pub fn new(
        domain: &str,
        publisher_id: &str,
        acc_relation: AccountRelation,
        cert_authority: Option<String>,
    ) -> Self {
        Self {
            domain: domain.to_string(),
            publisher_id: publisher_id.to_string(),
            acc_relation,
            cert_authority,
        }
    }

    pub fn parse(record_text: &str) -> Result<DataRecord> {
        let fields: Vec<&str> = record_text.split(',').collect();

        match fields.len() {
            3 => Ok(DataRecord {
                domain: fields[0].trim().to_string(),
                publisher_id: fields[1].trim().to_string(),
                acc_relation: AccountRelation::parse(fields[2])?,
                cert_authority: None,
            }),
            4 => Ok(DataRecord {
                domain: fields[0].trim().to_string(),
                publisher_id: fields[1].trim().to_string(),
                acc_relation: AccountRelation::parse(fields[2])?,
                cert_authority: Some(fields[3].trim().to_string()),
            }),
            _ => ads_txt_error(&format!("Invalid data record: {}", record_text)),
        }
    }
}

impl Variable {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    pub fn parse(line: &str) -> Result<Variable> {
        let fields: Vec<&str> = line.split('=').collect();

        match fields.len() {
            2 => Ok(Variable {
                name: fields[0].trim().to_string(),
                value: fields[1].trim().to_string(),
            }),
            _ => ads_txt_error(&format!("Invalid variable record: {}", line)),
        }
    }
}

impl AdsTxt {
    #[inline]
    fn is_comment(line: &str) -> bool {
        line.starts_with("#")
    }

    pub fn parse(text: &str) -> Result<AdsTxt> {
        let mut records: Vec<DataRecord> = vec![];
        let mut variables: Vec<Variable> = vec![];

        for line in text.lines() {
            if Self::is_comment(line) {
                continue;
            }

            if let Ok(record) = DataRecord::parse(line) {
                records.push(record);
                continue;
            }

            if let Ok(variable) = Variable::parse(line) {
                variables.push(variable);
                continue;
            }

            return ads_txt_error(&format!("Invalid ads.txt line: {}", line));
        }

        Ok(AdsTxt { records, variables })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_account_relation() {
        assert_eq!(
            AccountRelation::parse("direct"),
            Ok(AccountRelation::Direct)
        );
        assert_eq!(
            AccountRelation::parse("reseller"),
            Ok(AccountRelation::Reseller)
        );
        assert_eq!(
            AccountRelation::parse("RESELLER"),
            Ok(AccountRelation::Reseller)
        );
        assert_eq!(
            AccountRelation::parse("DIRECT"),
            Ok(AccountRelation::Direct)
        );
        assert_eq!(
            AccountRelation::parse("DIrecT"),
            Ok(AccountRelation::Direct)
        );
        assert_eq!(
            AccountRelation::parse("REsellER"),
            Ok(AccountRelation::Reseller)
        );
    }

    #[test]
    fn parsing_data_records() {
        assert_eq!(
            DataRecord::parse(""),
            ads_txt_error("Invalid data record: ")
        );
        assert_eq!(
            DataRecord::parse("greenadexchange.com, 12345, DIRECT, d75815a79"),
            Ok(DataRecord::new(
                "greenadexchange.com",
                "12345",
                AccountRelation::Direct,
                Some("d75815a79".to_string())
            ))
        );

        assert_eq!(
            DataRecord::parse("blueadexchange.com, XF436, DIRECT"),
            Ok(DataRecord::new(
                "blueadexchange.com",
                "XF436",
                AccountRelation::Direct,
                None
            ))
        )
    }

    #[test]
    fn parsing_variable_records() {
        assert_eq!(
            Variable::parse(""),
            ads_txt_error("Invalid variable record: ")
        );
        assert_eq!(
            Variable::parse("subdomain=divisionone.example.com"),
            Ok(Variable::new("subdomain", "divisionone.example.com"))
        );
    }
}
