use crate::AccountRelation::{Direct, Reseller};
use std::fmt::{Error, Formatter};

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

pub struct AdsTxt {
    pub records: Vec<DataRecord>,
    pub variables: Vec<Variable>,
}

pub type Variable = (String, String);

#[derive(Debug, Eq, PartialEq)]
pub enum AccountRelation {
    Direct,
    Reseller,
}

impl AccountRelation {
    fn parse(text: &str) -> Result<AccountRelation> {
        let relation = text.to_lowercase();

        if &relation == "direct" {
            Ok(Direct)
        } else if &relation == "reseller" {
            Ok(Reseller)
        } else {
            ads_txt_error(&format!("Invalid account relation: {}", text))
        }
    }
}

pub struct DataRecord {
    pub domain: String,
    pub publisher_id: String,
    pub acc_relation: AccountRelation,
    pub cert_authority: Option<String>,
}

impl DataRecord {
    pub fn parse(record_text: &str) -> Result<DataRecord> {
        let fields: Vec<&str> = record_text.split(',').collect();

        match fields.len() {
            3 => Ok(DataRecord {
                domain: fields[0].to_string(),
                publisher_id: fields[1].to_string(),
                acc_relation: AccountRelation::parse(fields[2])?,
                cert_authority: None,
            }),
            4 => Ok(DataRecord {
                domain: fields[0].to_string(),
                publisher_id: fields[1].to_string(),
                acc_relation: AccountRelation::parse(fields[2])?,
                cert_authority: None,
            }),
            _ => ads_txt_error(&format!("Invalid data record line: {}", record_text)),
        }
    }
}

impl AdsTxt {
    fn parse(text: &str) -> Result<AdsTxt> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::AccountRelation;

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
        unimplemented!();
    }
}
