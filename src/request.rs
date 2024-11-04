use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmeterData {
    #[serde(rename = "ServiceKey")]
    service_key: String,
    #[serde(rename = "message")]
    message: String,
    #[serde(rename = "statusCode")]
    status_code: String,
}

pub async fn get_ammeter(num: u32) -> Result<Option<i32>, String> {
    dbg!(num);
    let response = Client::new()
        .post("http://fspapp.ustb.edu.cn/app.GouDian/index.jsp?m=alipay&c=AliPay&a=getDbYe")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("DBNum={}", num))
        .send()
        .await
        .map_err(|err| err.to_string())?;
    let res_text = response.text().await.map_err(|err| err.to_string())?;
    let ammeter_data: AmmeterData =
        serde_json::from_str(&res_text).map_err(|err| err.to_string())?;
    if let Ok(kwh) = ammeter_data.service_key.parse::<i32>() {
        Ok(Some(kwh))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_ammeter() {
        let res = get_ammeter(11013200).await;
        dbg!(res.unwrap());
    }
}
