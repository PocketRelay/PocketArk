use crate::http::models::qos::{FireTypeQuery, FirewallQuery, QosQuery};
use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};
use hyper::{header, http::HeaderValue};
use log::debug;

pub struct RawXml(String);

impl IntoResponse for RawXml {
    fn into_response(self) -> Response {
        let mut resp = self.0.into_response();
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        );

        resp
    }
}

// Port local clients use for their qos server
const LOCAL_QOS_PORT: u16 = 42232;
const LOCAL_QOS_HOST: u32 = u32::from_be_bytes([127, 0, 0, 1]);

pub async fn qos_query(Query(query): Query<QosQuery>) -> RawXml {
    debug!("Redirected QOS query to local: {:?}", query);

    let response = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
    <qos>\
	<numprobes>0</numprobes>\
	<qosport>{}</qosport>\
	<probesize>0</probesize>\
	<qosip>{}</qosip>\
	<requestid>1</requestid>\
	<reqsecret>0</reqsecret>\
    </qos>",
        LOCAL_QOS_PORT, LOCAL_QOS_HOST
    );

    // host.docker.internal:54844 -> ec2-54-84-48-229.compute-1.amazonaws.com:https

    RawXml(response)
}

pub async fn qos_firewall(Query(query): Query<FirewallQuery>) -> RawXml {
    debug!("Redirected QOS firewall query to local: {:?}", query);

    // let ip1 = "159.153.230.30:17500";
    // let ip2 = "159.153.230.31:17501";
    // client tries to connect to one of the above ips (defined below)

    let response = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <firewall>
            <ips>
                <ips>{}</ips>
            </ips>
            <numinterfaces>1</numinterfaces>
            <ports>
                <ports>{}</ports>
            </ports>
            <requestid>1</requestid>
            <reqsecret>1</reqsecret>
        </firewall>"#,
        LOCAL_QOS_HOST, LOCAL_QOS_PORT,
    );

    RawXml(response)
}

pub async fn qos_firetype(Query(query): Query<FireTypeQuery>) -> RawXml {
    debug!("Redirected QOS type query to local: {:?}", query);

    let response = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <error>
            <component>16</component>
            <errorCode>13107216</errorCode>
            <errorName>QOS_ERR_INVALID_SLOT</errorName>
        </error>"#,
    );

    RawXml(response)
}
