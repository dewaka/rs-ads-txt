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

impl std::fmt::Display for AdsTxtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

fn ads_txt_error<T>(message: &str) -> Result<T> {
    Err(Box::new(AdsTxtError::new(message)))
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DataRecord {
    /// Domain for which the ads configuration applies
    pub domain: String,
    /// Publisher id
    pub publisher_id: String,
    /// Account relation
    pub acc_relation: AccountRelation,
    /// Optional cert authority
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
            domain: domain.trim().to_string(),
            publisher_id: publisher_id.trim().to_string(),
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Variable {
    pub name: String,
    pub value: String,
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

#[derive(Debug, Eq, PartialEq)]
pub struct AdsTxt {
    pub records: Vec<DataRecord>,
    pub variables: Vec<Variable>,
}

impl AdsTxt {
    #[inline]
    fn is_comment(line: &str) -> bool {
        line.starts_with("#")
    }

    pub fn new(records: &[DataRecord], variables: &[Variable]) -> Self {
        AdsTxt {
            records: records.to_vec(),
            variables: variables.to_vec(),
        }
    }

    pub fn empty() -> Self {
        Self::new(&[], &[])
    }

    pub fn parse(text: &str) -> Result<AdsTxt> {
        let mut records: Vec<DataRecord> = vec![];
        let mut variables: Vec<Variable> = vec![];

        for line in text.lines() {
            let line = line.trim_start();

            if line.is_empty() || Self::is_comment(line) {
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

    /// Parses ads.txt file leniently
    pub fn parse_lenient(text: &str) -> (AdsTxt, Vec<AdsTxtError>) {
        let mut records: Vec<DataRecord> = vec![];
        let mut variables: Vec<Variable> = vec![];
        let mut errors: Vec<AdsTxtError> = vec![];

        for line in text.lines() {
            let line = line.trim_start();

            if line.is_empty() || Self::is_comment(line) {
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

            errors.push(AdsTxtError::new(&format!("Invalid ads.txt line: {}", line)));
        }

        (AdsTxt { records, variables }, errors)
    }

    pub fn values(&self, name: &str) -> Vec<String> {
        let mut values = vec![];

        for v in &self.variables {
            if &v.name == name {
                values.push(v.value.to_string());
            }
        }

        values
    }

    pub fn sub_domains(&self) -> Vec<String> {
        let mut sub_domains = vec![];

        for v in &self.variables {
            if v.name.eq_ignore_ascii_case("subdomain") {
                sub_domains.push(v.value.to_string());
            }
        }

        sub_domains
    }

    pub fn contacts(&self) -> Vec<String> {
        let mut sub_domains = vec![];

        for v in &self.variables {
            if v.name.eq_ignore_ascii_case("contact") {
                sub_domains.push(v.value.to_string());
            }
        }

        sub_domains
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
        assert_eq!(
            Variable::parse("subdomain=   divisionone.example.com"),
            Ok(Variable::new("subdomain", "divisionone.example.com"))
        );
    }

    #[test]
    fn parsing_ads_txt() {
        let ads_txt1 = r"
        # ads.txt file for example.com:
        greenadexchange.com, 12345, DIRECT, d75815a79
        blueadexchange.com, XF436, DIRECT
        subdomain=divisionone.example.com
        ";

        let ads_txt2 = r"
        # ads.txt file for divisionone.example.com:
        silverssp.com, 5569, DIRECT, f496211
        orangeexchange.com, AB345, RESELLER
        ";

        // Should fail parsing strict
        let ads_txt3 = r"
        # ads.txt file for divisionone.example.com:
        silverssp.com, 5569
        orangeexchange.com, AB345, RESELLER
        ";

        let ads1 = AdsTxt::parse(ads_txt1);
        let ads2 = AdsTxt::parse(ads_txt2);
        let ads3 = AdsTxt::parse(ads_txt3);

        assert_eq!(
            ads1,
            Ok(AdsTxt::new(
                &[
                    DataRecord::new(
                        "greenadexchange.com",
                        "12345",
                        AccountRelation::Direct,
                        Some("d75815a79".to_string())
                    ),
                    DataRecord::new("blueadexchange.com", "XF436", AccountRelation::Direct, None),
                ],
                &[Variable::new("subdomain", "divisionone.example.com")],
            ))
        );

        assert_eq!(
            ads2,
            Ok(AdsTxt::new(
                &[
                    DataRecord::new(
                        "silverssp.com",
                        "5569",
                        AccountRelation::Direct,
                        Some("f496211".to_string())
                    ),
                    DataRecord::new(
                        "orangeexchange.com",
                        "AB345",
                        AccountRelation::Reseller,
                        None
                    ),
                ],
                &[],
            ))
        );

        assert_eq!(
            ads3,
            ads_txt_error("Invalid ads.txt line: silverssp.com, 5569")
        );

        assert_eq!(
            ads1.unwrap().values("subdomain"),
            vec!["divisionone.example.com".to_string()]
        );

        assert!(ads2.unwrap().values("subdomain").is_empty());
    }

    #[test]
    fn parsing_ads_txt_leniently() {
        // Should not fail parsing leniently
        let ads_txt3 = r"
        # ads.txt file for divisionone.example.com:
        silverssp.com, 5569
        orangeexchange.com, AB345, RESELLER
        ";

        let ads = AdsTxt::parse_lenient(ads_txt3);

        assert_eq!(
            ads,
            (
                AdsTxt::new(
                    &[DataRecord::new(
                        "orangeexchange.com",
                        "AB345",
                        AccountRelation::Reseller,
                        None
                    ),],
                    &[],
                ),
                vec![AdsTxtError::new(
                    "Invalid ads.txt line: silverssp.com, 5569"
                )]
            )
        );

        // Empty string should result in an empty AdsTxt and empty error messages list
        let ads2 = AdsTxt::parse_lenient("");
        assert_eq!(ads2, (AdsTxt::empty(), vec![]));
    }

    #[test]
    fn test_subdomains_retrieval() {
        let ads_txt = r"greenadexchange.com, 12345, DIRECT, d75815a79
            blueadexchange.com, XF436, DIRECT
            subdomain=divisionone.example.com";

        let ads = AdsTxt::parse(ads_txt);
        assert!(ads.is_ok());
        assert_eq!(ads.unwrap().sub_domains(), vec!("divisionone.example.com"));

        // We should get the same results when parsing leniently
        let (ads, errors) = AdsTxt::parse_lenient(ads_txt);
        assert_eq!(ads.sub_domains(), vec!("divisionone.example.com"));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_contacts_retrieval() {
        let ads_txt = r"# ads.txt file for example.com:
            greenadexchange.com, 12345, DIRECT, d75815a79
            blueadexchange.com, XF436, DIRECT
            contact=adops@example.com
            contact=http://example.com/contact-u";

        let ads = AdsTxt::parse(ads_txt);
        assert!(ads.is_ok());
        assert_eq!(
            ads.unwrap().contacts(),
            vec!("adops@example.com", "http://example.com/contact-u")
        );

        // We should get the same results when parsing leniently
        let (ads, errors) = AdsTxt::parse_lenient(ads_txt);
        assert_eq!(
            ads.contacts(),
            vec!("adops@example.com", "http://example.com/contact-u")
        );
        assert!(errors.is_empty());
    }
}
