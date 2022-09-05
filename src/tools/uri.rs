#[derive(Debug, PartialEq)]
pub enum ApiType {
    API,
    APIS,
}

// 先实现一版简单的，后面可以用yacc/nom库实现直接模式解析
#[derive(Debug, PartialEq)]
pub struct Uri {
    service: String,
    api_type: ApiType,             //api|apis
    api_group: String,             // ddx2x.github.io
    api_version: String,           // v1
    ns: Option<String>,            // dd
    resource_type: String,         // gps
    resource_name: Option<String>, // 123
    op: Option<String>,            //
}

impl Default for Uri {
    fn default() -> Self {
        Self {
            service: Default::default(),
            api_type: ApiType::API,
            api_group: Default::default(),
            api_version: Default::default(),
            ns: Default::default(),
            resource_type: Default::default(),
            resource_name: Default::default(),
            op: Default::default(),
        }
    }
}

pub fn parse(url: &str) -> Uri {
    let mut uri = Uri::default();
    let mut index = 0;

    for item in url.split('/') {
        match index {
            0 => {}
            1 => uri.service = item.into(),
            2 => {
                uri.api_type = if item == "api" {
                    ApiType::API
                } else {
                    ApiType::APIS
                }
            }
            3 => uri.api_group = item.into(),
            4 => uri.api_version = item.into(),
            5 => {
                if item != "namespaces" {
                    uri.resource_type = item.into();
                }
            }
            6 => {
                if uri.resource_type == "" {
                    uri.ns = Some(item.into());
                } else {
                    uri.resource_type = item.into()
                }
            }
            7 => {
                if uri.resource_type == "" {
                    uri.resource_type = item.into()
                } else {
                    uri.resource_name = Some(item.into())
                }
            }
            8 => {
                if uri.resource_name.is_none() {
                    uri.resource_name = Some(item.into())
                }
            }
            9 => {}
            10 => uri.op = Some(item.into()),
            _ => {}
        };
        index += 1;
    }
    uri
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_url() {
        let url = "/workload/apis/ddx2x.github.dev/v1/namespaces/abc/gps/0123321";

        assert_eq!(
            parse(url),
            Uri {
                service: "workload".into(),
                api_type: ApiType::APIS,
                api_group: "ddx2x.github.dev".into(),
                api_version: "v1".into(),
                ns: Some("abc".into()),
                resource_type: "gps".into(),
                resource_name: Some("0123321".into()),
                op: None
            }
        );

        let url = "/workload/apis/ddx2x.github.dev/v1/namespaces/abc/gps/0123321/op/watch";

        assert_eq!(
            parse(url),
            Uri {
                service: "workload".into(),
                api_type: ApiType::APIS,
                api_group: "ddx2x.github.dev".into(),
                api_version: "v1".into(),
                ns: Some("abc".into()),
                resource_type: "gps".into(),
                resource_name: Some("0123321".into()),
                op: Some("watch".into())
            }
        );
    }
}
