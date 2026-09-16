use benthic_protocol::messages::ui::login_error::{LoginError, Reason};
use benthic_protocol::messages::ui::login_event::Login;
use metaverse_messages::http::login::login_response::{LoginResponse, LoginStatus};
use metaverse_messages::http::login::simulator_login_protocol::SimulatorLoginProtocol;

/// send the login to simulator xml-rpc request
pub async fn login_to_simulator(
    login: Login,
) -> Result<(LoginResponse, std::net::IpAddr), LoginError> {
    let url = login.url.clone();

    // determine the IP the login was sent on. This needs to be stored in order to properly send
    // the CircuitCode UDP packet. The server checks to ensure that the IP of the CircuitCode
    // packet is the same as the IP that send the login.
    // Determine local IP used for login
    let local_ip = {
        let parsed_url = url::Url::parse(&url).map_err(|e| LoginError {
            reason: Reason::Connection,
            message: format!("Invalid URL: {:?}", e),
        })?;

        let host = parsed_url.host_str().ok_or(LoginError {
            reason: Reason::Connection,
            message: "URL has no host".into(),
        })?;

        let port = parsed_url.port_or_known_default().ok_or(LoginError {
            reason: Reason::Connection,
            message: "URL has no port".into(),
        })?;

        // Open a temporary TCP connection to get local IP
        let stream = std::net::TcpStream::connect((host, port)).map_err(|e| LoginError {
            reason: Reason::Connection,
            message: format!("TCP connect failed: {:?}", e),
        })?;

        stream
            .local_addr()
            .map(|addr| addr.ip())
            .map_err(|e| LoginError {
                reason: Reason::Connection,
                message: format!("Failed to get local IP: {:?}", e),
            })?
    };

    let client = awc::Client::default();
    // Serialize login data to XML-RPC
    let xml = SimulatorLoginProtocol::new(login).to_xmlrpc();
    // Send POST request
    let mut response = client
        .post(&url)
        .insert_header(("Content-Type", "text/xml; charset=utf-8"))
        .send_body(xml)
        .await
        .map_err(|e| LoginError {
            reason: Reason::Connection,
            message: format!("{:?}", e),
        })?;

    let body_bytes = response.body().await.map_err(|e| LoginError {
        reason: Reason::Connection,
        message: format!("{:?}", e),
    })?;

    let xml_string = String::from_utf8(body_bytes.to_vec()).unwrap();
    // Parse XML-RPC response

    match LoginResponse::from_xmlrpc(&xml_string) {
        Ok(login) => match login {
            LoginStatus::Success(success) => Ok((*success, local_ip)),
            LoginStatus::Failure(failure) => Err(failure),
        },
        Err(e) => Err(e)?,
    }
}
