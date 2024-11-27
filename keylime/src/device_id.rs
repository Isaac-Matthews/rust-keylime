use crate::{
    crypto::{self, CryptoError},
    tpm::{self, IAKResult, IDevIDResult, TpmError},
};
use log::*;
use openssl::x509::X509;
use std::path::Path;
use thiserror::Error;
use tss_esapi::{
    handles::KeyHandle,
    interface_types::algorithm::{AsymmetricAlgorithm, HashingAlgorithm},
    structures::{Attest, Data, Signature},
};

#[derive(Error, Debug)]
pub enum DeviceIDBuilderError {
    /// Failed to load certificate
    #[error("Failed to load certificate")]
    CertLoad(#[source] crypto::CryptoError),

    /// Public key does not match the certificate
    #[error("Public key does not match the certificate")]
    CertPubKeyMismatch(#[source] tpm::TpmError),

    /// Failed to create IAK
    #[error("Failed to create IAK")]
    IAKCreate(#[source] TpmError),

    /// Could not get IAK from provided handle
    #[error("Could not get IAK from provided handle")]
    IAKFromHandle(#[source] TpmError),

    /// IAK handle not set in DeviceIDBuilder
    #[error("IAK handle not set in DeviceBuilder. Set the IAK handle with the iak_handle() method from the DeviceIDBuilder object")]
    IAKHandleNotSet,

    /// IAK certificate not set in DeviceIDBuilder
    #[error("IAK certificate not set in DeviceBuilder. Set the IAK certificate with the iak_cert() method from the DeviceIDBuilder object")]
    IAKCertNotSet,

    /// IAK password not set in DeviceIDBuilder
    #[error("IAK password not set in DeviceBuilder. Set the IAK password with the iak_password() method from the DeviceIDBuilder object")]
    IAKPasswordNotSet,

    /// Failed to create IDevID
    #[error("Failed to create IDevID")]
    IDevIDCreate(#[source] TpmError),

    /// Could not get IDevID from provided handle
    #[error("Could not get IDevID from provided handle")]
    IDevIDFromHandle(#[source] TpmError),

    /// IDevID handle not set in DeviceIDBuilder
    #[error("IDevID handle not set in DeviceBuilder. Set the IDevID handle with the idevid_handle() method from the DeviceIDBuilder object")]
    IDevIDHandleNotSet,

    /// IDevID certificate not set in DeviceIDBuilder
    #[error("IDevID certificate not set in DeviceBuilder. Set the IDevID certificate with the idevid_cert() method from the DeviceIDBuilder object")]
    IDevIDCertNotSet,

    /// IDevID password not set in DeviceIDBuilder
    #[error("IDevID password not set in DeviceBuilder. Set the IDevID password with the idevid_password() method from the DeviceIDBuilder object")]
    IDevIDPasswordNotSet,

    /// Failed to get template from certificate
    #[error("Failed to get template from certificate")]
    TemplateFromCert(#[source] CryptoError),

    /// Failed to obtain template
    #[error("Failed to obtain template")]
    Template(#[source] TpmError),
}

#[derive(Error, Debug)]
pub enum DeviceIDError {
    /// Failed to certify
    #[error("Failed to certify credential with IAK")]
    Certify(#[source] TpmError),
}

#[derive(Debug, Default)]
pub struct DeviceIDBuilder<'a> {
    iak_handle: Option<&'a str>,
    iak_password: Option<&'a str>,
    iak_cert: Option<X509>,
    iak_cert_path: Option<&'a str>,
    iak_template: Option<&'a str>,
    iak_asym_alg: Option<&'a str>,
    iak_hash_alg: Option<&'a str>,
    idevid_handle: Option<&'a str>,
    idevid_password: Option<&'a str>,
    idevid_cert: Option<X509>,
    idevid_cert_path: Option<&'a str>,
    idevid_template: Option<&'a str>,
    idevid_asym_alg: Option<&'a str>,
    idevid_hash_alg: Option<&'a str>,
}

impl<'a> DeviceIDBuilder<'a> {
    /// Create a new DeviceIDBuilder object
    pub fn new() -> DeviceIDBuilder<'a> {
        Self::default()
    }

    /// Set the IAK handle to use when building the DeviceID object
    ///
    /// # Arguments:
    ///
    /// * iak_handle (&str): The IAK handle
    pub fn iak_handle(mut self, iak_handle: &'a str) -> DeviceIDBuilder<'a> {
        self.iak_handle = Some(iak_handle);
        self
    }

    /// Set the IAK password to use when building the DeviceID object
    ///
    /// # Arguments:
    ///
    /// * iak_password (&str): The IAK password
    pub fn iak_password(
        mut self,
        iak_password: &'a str,
    ) -> DeviceIDBuilder<'a> {
        self.iak_password = Some(iak_password);
        self
    }

    /// Set the path to the IAK certificate to use when building the DeviceID object
    ///
    /// # Arguments:
    ///
    /// * path (&str): The path to the IAK certificate
    pub fn iak_cert_path(mut self, path: &'a str) -> DeviceIDBuilder<'a> {
        self.iak_cert_path = Some(path);
        self
    }

    /// Set the template to use for the IAK
    ///
    /// If the string is empty, or the special keywords 'detect' or 'default' are set, the template
    /// will be obtained from the IAK certificate provided in `iak_cert`
    ///
    /// # Arguments:
    ///
    /// * template(&str): The template name to use.
    pub fn iak_template(mut self, template: &'a str) -> DeviceIDBuilder<'a> {
        self.iak_template = Some(template);
        self
    }

    /// Set the Asymmetric algorithm to use in the IAK template
    ///
    /// # Arguments:
    ///
    /// * iak_asym_alg (&str): The template Asymmetric algorithm to use for the IAK template
    pub fn iak_asym_alg(mut self, asym_alg: &'a str) -> DeviceIDBuilder<'a> {
        self.iak_asym_alg = Some(asym_alg);
        self
    }

    /// Set the template Hash algorithm to use in the IAK template
    ///
    /// # Arguments:
    ///
    /// * iak_hash_alg (&str): The Hash algorithm to use for the IAK template
    pub fn iak_hash_alg(mut self, hash_alg: &'a str) -> DeviceIDBuilder<'a> {
        self.iak_hash_alg = Some(hash_alg);
        self
    }

    /// Set the IDevID handle to use when building the DeviceID object
    ///
    /// # Arguments:
    ///
    /// * idevid_handle (&str): The IDevID handle
    pub fn idevid_handle(
        mut self,
        idevid_handle: &'a str,
    ) -> DeviceIDBuilder<'a> {
        self.idevid_handle = Some(idevid_handle);
        self
    }

    /// Set the IDevID password to use when building the DeviceID object
    ///
    /// # Arguments:
    ///
    /// * idevid_password (&str): The password to use for the IDevID
    pub fn idevid_password(
        mut self,
        idevid_password: &'a str,
    ) -> DeviceIDBuilder<'a> {
        self.idevid_password = Some(idevid_password);
        self
    }

    /// Set the IDevID certificate path to use when building the DeviceID object
    ///
    /// # Arguments:
    ///
    /// * path (&str): The path to the IDevID certificate
    pub fn idevid_cert_path(mut self, path: &'a str) -> DeviceIDBuilder<'a> {
        self.idevid_cert_path = Some(path);
        self
    }

    /// Set the template to use for the IDevID
    ///
    /// If the string is empty, or the special keywords 'detect' or 'default' are set, the template
    /// will be obtained from the IDevID certificate provided in `idevid_cert`
    ///
    /// # Arguments:
    ///
    /// * template(&str): The template name to use.
    pub fn idevid_template(
        mut self,
        template: &'a str,
    ) -> DeviceIDBuilder<'a> {
        self.idevid_template = Some(template);
        self
    }

    /// Set the Asymmetric algorithm to use in the IDevID template
    ///
    /// # Arguments:
    ///
    /// * idevid_asym_alg (&str): The template Asymmetric algorithm to use for the IDevID template
    pub fn idevid_asym_alg(
        mut self,
        asym_alg: &'a str,
    ) -> DeviceIDBuilder<'a> {
        self.idevid_asym_alg = Some(asym_alg);
        self
    }

    /// Set the template Hash algorithm to use in the IDevID template
    ///
    /// # Arguments:
    ///
    /// * idevid_hash_alg (&str): The Hash algorithm to use for the IDevID template
    pub fn idevid_hash_alg(
        mut self,
        hash_alg: &'a str,
    ) -> DeviceIDBuilder<'a> {
        self.idevid_hash_alg = Some(hash_alg);
        self
    }

    /// Get the IAK template
    ///
    /// If configured to detect, get the template from the IAK certificate
    /// Otherwise, construct the template from the provided algorithms
    fn get_iak_template(
        &mut self,
    ) -> Result<(AsymmetricAlgorithm, HashingAlgorithm), DeviceIDBuilderError>
    {
        let iak_cert = self.get_iak_cert()?;
        let detected = &crypto::match_cert_to_template(iak_cert)
            .map_err(DeviceIDBuilderError::TemplateFromCert)?;

        let template = self.iak_template.unwrap_or("").trim();
        let asym_alg = self.iak_asym_alg.unwrap_or("").trim();
        let hash_alg = self.iak_hash_alg.unwrap_or("").trim();

        tpm::get_idevid_template(detected, template, asym_alg, hash_alg)
            .map_err(DeviceIDBuilderError::Template)
    }

    /// Get the IDevID template
    ///
    /// If configured to detect, get the template from the IDevID certificate
    /// Otherwise, construct the template from the provided algorithms
    fn get_idevid_template(
        &mut self,
    ) -> Result<(AsymmetricAlgorithm, HashingAlgorithm), DeviceIDBuilderError>
    {
        let idevid_cert = self.get_idevid_cert()?;
        let detected = &crypto::match_cert_to_template(idevid_cert)
            .map_err(DeviceIDBuilderError::TemplateFromCert)?;

        let template = self.idevid_template.unwrap_or("").trim();
        let asym_alg = self.idevid_asym_alg.unwrap_or("").trim();
        let hash_alg = self.idevid_hash_alg.unwrap_or("").trim();

        tpm::get_idevid_template(detected, template, asym_alg, hash_alg)
            .map_err(DeviceIDBuilderError::Template)
    }

    /// Get the IAK from the given handle and password or recreate following the given template
    ///
    /// If a handle has been set, try to obtain the IAK from the handle
    /// If there is a configured IAK password, add the password to the handle
    ///
    /// If the handle is empty, recreate the IAK using the provided algorithms
    fn get_iak(
        &mut self,
        tpm_ctx: &mut tpm::Context,
    ) -> Result<IAKResult, DeviceIDBuilderError> {
        let (asym_alg, hash_alg) = self.get_iak_template()?;
        match self.iak_handle {
            Some(handle) => {
                if handle.trim().is_empty() {
                    info!("Recreating IAK.");
                    tpm_ctx
                        .create_iak(asym_alg, hash_alg)
                        .map_err(DeviceIDBuilderError::IAKCreate)
                } else {
                    let password = self.iak_password.unwrap_or("").trim();
                    info!("Collecting persisted IAK.");
                    tpm_ctx
                        .iak_from_handle(handle.trim(), password)
                        .map_err(DeviceIDBuilderError::IAKFromHandle)
                }
            }
            None => Err(DeviceIDBuilderError::IAKHandleNotSet),
        }
    }

    /// Get the IDevID from the given handle and password or recreate following the given template
    ///
    /// If a handle has been set, try to obtain the IDevID from the handle
    /// If there is a configured IDevID password, add the password to the handle
    ///
    /// If the handle is empty, recreate the IDevId using the provided algorithms
    fn get_idevid(
        &mut self,
        tpm_ctx: &mut tpm::Context,
    ) -> Result<IDevIDResult, DeviceIDBuilderError> {
        let (asym_alg, hash_alg) = self.get_idevid_template()?;
        match self.idevid_handle {
            Some(handle) => {
                if handle.trim().is_empty() {
                    info!("Recreating IDevID.");
                    tpm_ctx
                        .create_idevid(asym_alg, hash_alg)
                        .map_err(DeviceIDBuilderError::IDevIDCreate)
                } else {
                    let password = self.idevid_password.unwrap_or("").trim();
                    info!("Collecting persisted IDevID.");
                    tpm_ctx
                        .idevid_from_handle(handle.trim(), password.trim())
                        .map_err(DeviceIDBuilderError::IDevIDFromHandle)
                }
            }
            None => Err(DeviceIDBuilderError::IDevIDHandleNotSet),
        }
    }

    /// Get the IAK certificate
    /// If the iak_cert is not set, try to load the certificate from the iak_cert_path and cache
    /// the loaded certificate
    fn get_iak_cert(&mut self) -> Result<&X509, DeviceIDBuilderError> {
        match self.iak_cert {
            Some(ref cert) => Ok(cert),
            None => match &self.iak_cert_path {
                Some(path) => {
                    if path.trim().is_empty() {
                        debug!(
                                "The IAK certificate was not set in the configuration file"
                            );
                        Err(DeviceIDBuilderError::IAKCertNotSet)
                    } else {
                        self.iak_cert = Some(crypto::load_x509(Path::new(path.trim())).map_err(|e| {
                                debug!("Could not load IAK certificate from {path}: {e}");
                                e
                            }).map_err(DeviceIDBuilderError::CertLoad)?);
                        if let Some(ref cert) = &self.iak_cert {
                            Ok(cert)
                        } else {
                            unreachable!();
                        }
                    }
                }
                None => Err(DeviceIDBuilderError::IAKCertNotSet),
            },
        }
    }

    /// Get the IDevID certificate
    ///
    /// If the idevid_cert is not set, try to load the certificate from the idevid_cert_path and cache the loaded certificate
    fn get_idevid_cert(&mut self) -> Result<&X509, DeviceIDBuilderError> {
        match self.idevid_cert {
            Some(ref cert) => Ok(cert),
            None => match self.idevid_cert_path {
                Some(path) => {
                    if path.trim().is_empty() {
                        debug!(
                                "The IDevId certificate was not set in the configuration file"
                            );
                        Err(DeviceIDBuilderError::IDevIDCertNotSet)
                    } else {
                        self.idevid_cert = Some(crypto::load_x509(Path::new(path.trim())).map_err(|e| {
                                debug!("Could not load IAK certificate from {path}: {e}");
                                e
                            }).map_err(DeviceIDBuilderError::CertLoad)?);
                        if let Some(ref cert) = self.idevid_cert {
                            Ok(cert)
                        } else {
                            unreachable!();
                        }
                    }
                }
                None => Err(DeviceIDBuilderError::IDevIDCertNotSet),
            },
        }
    }

    /// Generate the DeviceID object using the previously set options
    pub fn build(
        mut self,
        tpm_ctx: &mut tpm::Context,
    ) -> Result<DeviceID, DeviceIDBuilderError> {
        let iak = self.get_iak(tpm_ctx)?;
        let Some(iak_cert) = self.iak_cert.take() else {
            unreachable!();
        };

        // Check that recreated/collected IAK key matches the one in the certificate
        tpm::check_pubkey_match_cert(&iak.public, &iak_cert, "IAK")
            .map_err(DeviceIDBuilderError::CertPubKeyMismatch)?;

        let idevid = self.get_idevid(tpm_ctx)?;
        let Some(idevid_cert) = self.idevid_cert.take() else {
            unreachable!();
        };

        // Check that recreated/collected IDevID key matches the one in the certificate
        tpm::check_pubkey_match_cert(&idevid.public, &idevid_cert, "IDevID")
            .map_err(DeviceIDBuilderError::CertPubKeyMismatch)?;

        Ok(DeviceID {
            iak,
            iak_cert,
            idevid,
            idevid_cert,
        })
    }
}

#[derive(Debug)]
pub struct DeviceID {
    pub iak: IAKResult,
    pub iak_cert: X509,
    pub idevid: IDevIDResult,
    pub idevid_cert: X509,
}

impl DeviceID {
    /// Certify IAK against AK using the provided qualifying data
    ///
    /// # Arguments:
    ///
    /// * qualifying_data (Data): The qualifying data.
    /// * ak (KeyHandle): The AK handle
    pub fn certify(
        &mut self,
        qualifying_data: Data,
        ak: KeyHandle,
        tpm_ctx: &mut tpm::Context,
    ) -> Result<(Attest, Signature), DeviceIDError> {
        tpm_ctx
            .certify_credential_with_iak(qualifying_data, ak, self.iak.handle)
            .map_err(DeviceIDError::Certify)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_id_builder_setting() {
        let _builder = DeviceIDBuilder::new()
            .iak_handle("")
            .iak_cert_path("")
            .iak_password("")
            .iak_template("")
            .iak_asym_alg("")
            .iak_hash_alg("")
            .idevid_handle("")
            .idevid_cert_path("")
            .idevid_password("")
            .idevid_template("")
            .idevid_asym_alg("")
            .idevid_hash_alg("");
    }

    #[tokio::test]
    #[cfg(feature = "testing")]
    async fn test_device_id_builder() {
        let _mutex = tpm::testing::lock_tests().await;
        let certs_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test-data")
            .join("iak-idevid-certs");

        if certs_dir.exists() {
            let iak_cert = certs_dir.join("iak.cert.pem");
            let idevid_cert = certs_dir.join("idevid.cert.pem");
            if iak_cert.exists() && idevid_cert.exists() {
                let mut tpm_ctx = tpm::Context::new().unwrap(); //#[allow_ci]
                let result = DeviceIDBuilder::new()
                    .iak_handle("")
                    .iak_cert_path(
                        iak_cert
                            .to_str()
                            .expect("Failed to get str for IAK cert"),
                    )
                    .iak_password("")
                    .iak_template("")
                    .iak_asym_alg("")
                    .iak_hash_alg("")
                    .idevid_handle("")
                    .idevid_cert_path(
                        idevid_cert
                            .to_str()
                            .expect("Failed to get str for IDevID cert"),
                    )
                    .idevid_password("")
                    .idevid_template("")
                    .idevid_asym_alg("")
                    .idevid_hash_alg("")
                    .build(&mut tpm_ctx);
                assert!(result.is_ok(), "Result: {result:?}");
                let dev_id = result.unwrap(); //#[allow_ci]

                // Flush context to free TPM memory
                let r = tpm_ctx.flush_context(dev_id.iak.handle.into());
                assert!(r.is_ok(), "Result: {r:?}");
                let r = tpm_ctx.flush_context(dev_id.idevid.handle.into());
                assert!(r.is_ok(), "Result: {r:?}");
            }
        }
    }
}
