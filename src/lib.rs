extern crate data_query_lexical;
#[macro_use]
extern crate data_query_proc;
extern crate jq_rs;
extern crate railsgun;
extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

mod error;

pub use crate::error::QueryError;
use data_query_lexical::{GenericObjectIndex, LexOperator, LexicalOperations, Slicer};

use serde::Serialize;
use serde_json::{Map, Value};
use std::cmp::Ordering;

/// Alias for a `Result` with the error type `serde_json::Error`.
pub type QueryResult<T> = std::result::Result<T, QueryError>;

struct ComType {
    usize: Option<usize>,
    string: Option<String>,
}

impl PartialOrd<Self> for ComType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::*;
        if let (Some(s), Some(o)) = (self.usize, other.usize) {
            if s > o {
                return Some(Greater);
            } else if s < o {
                return Some(Less);
            } else if s == o {
                return Some(Equal);
            }
        }
        None
    }
}

// impl Ord for ComType {
//     fn cmp(&self, other: &Self) -> Ordering {
//         todo!()
//     }
// }

impl Eq for ComType {}

impl PartialEq for ComType {
    fn eq(&self, other: &Self) -> bool {
        if let (Some(s), Some(o)) = (self.usize, other.usize) {
            s == o
        } else {
            false
        }
    }
}

impl From<usize> for ComType {
    fn from(u: usize) -> Self {
        Self {
            usize: Some(u),
            string: Some(format!("{}", u)),
        }
    }
}
impl From<&usize> for ComType {
    fn from(u: &usize) -> Self {
        Self {
            usize: Some(*u),
            string: Some(format!("{}", u)),
        }
    }
}

impl From<String> for ComType {
    fn from(s: String) -> Self {
        Self {
            usize: s.parse::<usize>().ok(),
            string: Some(s),
        }
    }
}

impl From<&str> for ComType {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

impl From<&mut usize> for ComType {
    fn from(u: &mut usize) -> Self {
        Self::from(u.clone())
    }
}

#[cfg(not(feature = "jq"))]
pub fn query<S: Serialize, Q: TryInto<LexicalOperations>>(s: S, query: Q) -> QueryResult<Value> {
    {
        let mut lexes = query
            .try_into()
            // TODO: This error handling needs to be fixed!
            .map_err(|_e| QueryError::from(format!("Gulp")))?;
        let data = serde_json::to_value(s).map_err(QueryError::from)?;
        let mut results = Vec::new();
        query_processor(&data, &mut lexes, &mut results, 0)?;
        Ok(Value::Array(results))
    }
}

#[cfg(feature = "jq")]
pub fn query<S: Serialize>(s: S, query: &str) -> QueryResult<Value> {
    let mut lexer = jq_rs::compile(query).map_err(QueryError::from)?;
    let data_string = serde_json::to_string(&s).map_err(QueryError::from)?;
    let json_value = &lexer.run(&data_string).map_err(QueryError::from)?;
    serde_json::from_str(json_value.trim()).map_err(QueryError::from)
}

fn query_processor(
    data: &Value,
    query: &mut LexicalOperations,
    results: &mut Vec<Value>,
    mut depth: usize,
) -> QueryResult<()> {
    depth += 1;
    if query.is_empty() {
        results.push(data.clone());
        return Ok(());
    } else {
        let key_query = query
            .pop_front()
            .ok_or(QueryError::UncontrolledError("Empty".to_string()))?;
        match data {
            Value::Array(v) => match key_query.clone() {
                LexOperator::Identifier(ident) => {
                    return if let Ok(i) = ident.parse::<usize>() {
                        query_processor(&v[i], query, results, depth)
                    } else {
                        Err(QueryError::CannotUseIdentifierAsArrayKeyIndex(ident))
                    };
                }
                LexOperator::Pipe(_p) => {
                    todo!();
                }
                LexOperator::Generic(mut g) => {
                    return query_slice_w_generic_object_index(&v, &mut g, query, results, depth);
                }
            },
            Value::Object(m) => match key_query {
                LexOperator::Identifier(ident) => {
                    return if m.contains_key(&ident) {
                        if let Some(value) = m.get(&*ident) {
                            query_processor(value, query, results, depth)
                        } else {
                            Ok(())
                        }
                    } else {
                        Err(QueryError::CannotUseIdentifierAsArrayKeyIndex(ident))
                    };
                }
                LexOperator::Pipe(_p) => {
                    todo!();
                }
                LexOperator::Generic(mut g) => {
                    return query_map_w_generic_object_index(m, &mut g, query, results, depth);
                }
            },
            _ => {
                return Ok(());
            }
        }
    }
    Ok(())
}

fn query_slice_w_generic_object_index(
    data: &Vec<Value>,
    index_match: &mut GenericObjectIndex,
    query: &mut LexicalOperations,
    results: &mut Vec<Value>,
    depth: usize,
) -> QueryResult<()> {
    for (k, v) in data.iter().enumerate() {
        if match_slice_to_key(&format!("{}", k), index_match) {
            query_processor(v, query, results, depth)?
        }
    }
    Ok(())
}

fn query_map_w_generic_object_index(
    data: &Map<String, Value>,
    index_match: &mut GenericObjectIndex,
    query: &mut LexicalOperations,
    results: &mut Vec<Value>,
    depth: usize,
) -> QueryResult<()> {
    for (k, v) in data.iter() {
        if match_slice_to_key(&format!("{}", k), index_match) {
            query_processor(v, query, results, depth)?
        }
    }
    Ok(())
}

fn key_match_map(key: &String, query: &LexOperator) -> bool {
    match query {
        LexOperator::Identifier(ident) => {
            if key == ident {
                true
            } else {
                false
            }
        }
        LexOperator::Pipe(_p) => {
            todo!();
        }
        LexOperator::Generic(g) => {
            return match_slice_to_key(key, &mut g.clone());
        }
    }
}

fn match_slice_to_key(key: &str, query: &mut GenericObjectIndex) -> bool {
    let key_comp: ComType = key.into();
    match query {
        GenericObjectIndex::Wildcard => true,
        GenericObjectIndex::Slice(slice) => {
            for s in slice {
                match s {
                    Slicer::Index(i) => {
                        if key_comp == ComType::from(i) {
                            return true;
                        }
                    }
                    Slicer::Slice(f, t) => {
                        if key_comp <= ComType::from(f) && key_comp >= ComType::from(t) {
                            return true;
                        }
                    }
                    Slicer::Ident(ident) => {
                        if let Ok(ref i) = ident.parse::<usize>() {
                            if key_comp == ComType::from(i) {
                                return true;
                            }
                        }
                    }
                }
            }
            return false;
        }
    }
}

// fn array_items(v: Value, lex: LexicalOperations) -> {
//
//     let mut tmp_value = Vec::new();
//     for (k, v) in v.into_iter().enumerate() {
//         if key_match_array(&k, &key_query) {
//             tmp_value =
//                 vec![tmp_value, query_processor(v, query).unwrap_or_default()].concat();
//         }
//     }
//     Ok(tmp_value)
// }

#[cfg(test)]
pub mod test {
    use crate::{query, ComType};
    use data_query_lexical::{compile, LexOperator};
    use serde_derive::Serialize;
    use serde_json::Value;
    use std::collections::{HashMap, LinkedList};
    use std::iter::Map;

    const TEST_OBJECT_RAW: &str = r##"{"apiVersion":"v1","kind":"Pod","metadata":{"annotations":{"kubectl.kubernetes.io/default-container":"wordpress","kubectl.kubernetes.io/default-logs-container":"wordpress","kubectl.kubernetes.io/restartedAt":"2022-06-07T20:38:55+09:00","prometheus.io/path":"/stats/prometheus","prometheus.io/port":"15020","prometheus.io/scrape":"true","sidecar.istio.io/status":"{\"initContainers\":[\"istio-init\"],\"containers\":[\"istio-proxy\"],\"volumes\":[\"istio-envoy\",\"istio-data\",\"istio-podinfo\",\"istio-token\",\"istiod-ca-cert\"],\"imagePullSecrets\":null,\"revision\":\"default\"}"},"creationTimestamp":"2022-06-07T11:38:55Z","generateName":"katsuoryuu-org-wordpress-b94d59c49-","labels":{"app.kubernetes.io/instance":"katsuoryuu-org","app.kubernetes.io/managed-by":"Helm","app.kubernetes.io/name":"wordpress","helm.sh/chart":"wordpress-13.1.1","pod-template-hash":"b94d59c49","security.istio.io/tlsMode":"istio","service.istio.io/canonical-name":"wordpress","service.istio.io/canonical-revision":"latest"},"managedFields":[{"apiVersion":"v1","fieldsType":"FieldsV1","fieldsV1":{"f:metadata":{"f:annotations":{".":{},"f:kubectl.kubernetes.io/restartedAt":{}},"f:generateName":{},"f:labels":{".":{},"f:app.kubernetes.io/instance":{},"f:app.kubernetes.io/managed-by":{},"f:app.kubernetes.io/name":{},"f:helm.sh/chart":{},"f:pod-template-hash":{}},"f:ownerReferences":{".":{},"k:{\"uid\":\"4a5f15a1-0380-4c48-9980-52beb6173eaa\"}":{}}},"f:spec":{"f:affinity":{".":{},"f:podAntiAffinity":{".":{},"f:preferredDuringSchedulingIgnoredDuringExecution":{}}},"f:containers":{"k:{\"name\":\"wordpress\"}":{".":{},"f:env":{".":{},"k:{\"name\":\"ALLOW_EMPTY_PASSWORD\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"APACHE_HTTPS_PORT_NUMBER\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"APACHE_HTTP_PORT_NUMBER\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"BITNAMI_DEBUG\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"MARIADB_HOST\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"MARIADB_PORT_NUMBER\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_AUTO_UPDATE_LEVEL\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_BLOG_NAME\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_DATABASE_NAME\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_DATABASE_PASSWORD\"}":{".":{},"f:name":{},"f:valueFrom":{".":{},"f:secretKeyRef":{}}},"k:{\"name\":\"WORDPRESS_DATABASE_USER\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_EMAIL\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_ENABLE_HTACCESS_PERSISTENCE\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_EXTRA_WP_CONFIG_CONTENT\"}":{".":{},"f:name":{}},"k:{\"name\":\"WORDPRESS_FIRST_NAME\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_HTACCESS_OVERRIDE_NONE\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_LAST_NAME\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_PASSWORD\"}":{".":{},"f:name":{},"f:valueFrom":{".":{},"f:secretKeyRef":{}}},"k:{\"name\":\"WORDPRESS_PLUGINS\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_SCHEME\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_SKIP_BOOTSTRAP\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_TABLE_PREFIX\"}":{".":{},"f:name":{},"f:value":{}},"k:{\"name\":\"WORDPRESS_USERNAME\"}":{".":{},"f:name":{},"f:value":{}}},"f:image":{},"f:imagePullPolicy":{},"f:livenessProbe":{".":{},"f:failureThreshold":{},"f:httpGet":{".":{},"f:path":{},"f:port":{},"f:scheme":{}},"f:initialDelaySeconds":{},"f:periodSeconds":{},"f:successThreshold":{},"f:timeoutSeconds":{}},"f:name":{},"f:ports":{".":{},"k:{\"containerPort\":8080,\"protocol\":\"TCP\"}":{".":{},"f:containerPort":{},"f:name":{},"f:protocol":{}},"k:{\"containerPort\":8443,\"protocol\":\"TCP\"}":{".":{},"f:containerPort":{},"f:name":{},"f:protocol":{}}},"f:readinessProbe":{".":{},"f:failureThreshold":{},"f:httpGet":{".":{},"f:path":{},"f:port":{},"f:scheme":{}},"f:initialDelaySeconds":{},"f:periodSeconds":{},"f:successThreshold":{},"f:timeoutSeconds":{}},"f:resources":{},"f:securityContext":{".":{},"f:runAsNonRoot":{},"f:runAsUser":{}},"f:terminationMessagePath":{},"f:terminationMessagePolicy":{},"f:volumeMounts":{".":{},"k:{\"mountPath\":\"/bitnami/wordpress\"}":{".":{},"f:mountPath":{},"f:name":{},"f:subPath":{}}}}},"f:dnsPolicy":{},"f:enableServiceLinks":{},"f:hostAliases":{".":{},"k:{\"ip\":\"127.0.0.1\"}":{".":{},"f:hostnames":{},"f:ip":{}}},"f:restartPolicy":{},"f:schedulerName":{},"f:securityContext":{".":{},"f:fsGroup":{}},"f:serviceAccount":{},"f:serviceAccountName":{},"f:terminationGracePeriodSeconds":{},"f:volumes":{".":{},"k:{\"name\":\"wordpress-data\"}":{".":{},"f:name":{},"f:persistentVolumeClaim":{".":{},"f:claimName":{}}}}}},"manager":"kube-controller-manager","operation":"Update","time":"2022-06-07T11:38:55Z"},{"apiVersion":"v1","fieldsType":"FieldsV1","fieldsV1":{"f:status":{"f:conditions":{"k:{\"type\":\"ContainersReady\"}":{".":{},"f:lastProbeTime":{},"f:lastTransitionTime":{},"f:status":{},"f:type":{}},"k:{\"type\":\"Initialized\"}":{".":{},"f:lastProbeTime":{},"f:lastTransitionTime":{},"f:status":{},"f:type":{}},"k:{\"type\":\"Ready\"}":{".":{},"f:lastProbeTime":{},"f:lastTransitionTime":{},"f:status":{},"f:type":{}}},"f:containerStatuses":{},"f:hostIP":{},"f:initContainerStatuses":{},"f:phase":{},"f:podIP":{},"f:podIPs":{".":{},"k:{\"ip\":\"172.17.0.27\"}":{".":{},"f:ip":{}}},"f:startTime":{}}},"manager":"kubelet","operation":"Update","subresource":"status","time":"2022-06-07T11:44:07Z"}],"name":"katsuoryuu-org-wordpress-b94d59c49-csvzr","namespace":"wordpress","ownerReferences":[{"apiVersion":"apps/v1","blockOwnerDeletion":true,"controller":true,"kind":"ReplicaSet","name":"katsuoryuu-org-wordpress-b94d59c49","uid":"4a5f15a1-0380-4c48-9980-52beb6173eaa"}],"resourceVersion":"77663255","uid":"8f03f916-4fd1-462f-a52c-0041b411179f"},"spec":{"affinity":{"podAntiAffinity":{"preferredDuringSchedulingIgnoredDuringExecution":[{"podAffinityTerm":{"labelSelector":{"matchLabels":{"app.kubernetes.io/instance":"katsuoryuu-org","app.kubernetes.io/name":"wordpress"}},"namespaces":["wordpress"],"topologyKey":"kubernetes.io/hostname"},"weight":1}]}},"containers":[{"env":[{"name":"BITNAMI_DEBUG","value":"false"},{"name":"ALLOW_EMPTY_PASSWORD","value":"yes"},{"name":"MARIADB_HOST","value":"mariadb.database-mysql.svc.cluster.local"},{"name":"MARIADB_PORT_NUMBER","value":"3306"},{"name":"WORDPRESS_DATABASE_NAME","value":"katsuoryuu.org"},{"name":"WORDPRESS_DATABASE_USER","value":"katsuoryuu.org"},{"name":"WORDPRESS_DATABASE_PASSWORD","valueFrom":{"secretKeyRef":{"key":"mariadb-password","name":"katsuoryuu-org-wordpress-externaldb"}}},{"name":"WORDPRESS_USERNAME","value":"user"},{"name":"WORDPRESS_PASSWORD","valueFrom":{"secretKeyRef":{"key":"wordpress-password","name":"katsuoryuu-org-wordpress"}}},{"name":"WORDPRESS_EMAIL","value":"user@example.com"},{"name":"WORDPRESS_FIRST_NAME","value":"FirstName"},{"name":"WORDPRESS_LAST_NAME","value":"LastName"},{"name":"WORDPRESS_HTACCESS_OVERRIDE_NONE","value":"no"},{"name":"WORDPRESS_ENABLE_HTACCESS_PERSISTENCE","value":"no"},{"name":"WORDPRESS_BLOG_NAME","value":"User's Blog!"},{"name":"WORDPRESS_SKIP_BOOTSTRAP","value":"no"},{"name":"WORDPRESS_TABLE_PREFIX","value":"wp_"},{"name":"WORDPRESS_SCHEME","value":"http"},{"name":"WORDPRESS_EXTRA_WP_CONFIG_CONTENT"},{"name":"WORDPRESS_AUTO_UPDATE_LEVEL","value":"none"},{"name":"WORDPRESS_PLUGINS","value":"none"},{"name":"APACHE_HTTP_PORT_NUMBER","value":"8080"},{"name":"APACHE_HTTPS_PORT_NUMBER","value":"8443"}],"image":"docker.io/bitnami/wordpress:5.9.2-debian-10-r4","imagePullPolicy":"IfNotPresent","livenessProbe":{"failureThreshold":6,"httpGet":{"path":"/app-health/wordpress/livez","port":15020,"scheme":"HTTP"},"initialDelaySeconds":120,"periodSeconds":10,"successThreshold":1,"timeoutSeconds":5},"name":"wordpress","ports":[{"containerPort":8080,"name":"http","protocol":"TCP"},{"containerPort":8443,"name":"https","protocol":"TCP"}],"readinessProbe":{"failureThreshold":6,"httpGet":{"path":"/app-health/wordpress/readyz","port":15020,"scheme":"HTTP"},"initialDelaySeconds":30,"periodSeconds":10,"successThreshold":1,"timeoutSeconds":5},"resources":{},"securityContext":{"runAsNonRoot":true,"runAsUser":1001},"terminationMessagePath":"/dev/termination-log","terminationMessagePolicy":"File","volumeMounts":[{"mountPath":"/bitnami/wordpress","name":"wordpress-data","subPath":"wordpress"},{"mountPath":"/var/run/secrets/kubernetes.io/serviceaccount","name":"kube-api-access-r74bw","readOnly":true}]},{"args":["proxy","sidecar","--domain","$(POD_NAMESPACE).svc.cluster.local","--proxyLogLevel=warning","--proxyComponentLogLevel=misc:error","--log_output_level=default:info","--concurrency","2"],"env":[{"name":"JWT_POLICY","value":"third-party-jwt"},{"name":"PILOT_CERT_PROVIDER","value":"istiod"},{"name":"CA_ADDR","value":"istiod.istio-system.svc:15012"},{"name":"POD_NAME","valueFrom":{"fieldRef":{"apiVersion":"v1","fieldPath":"metadata.name"}}},{"name":"POD_NAMESPACE","valueFrom":{"fieldRef":{"apiVersion":"v1","fieldPath":"metadata.namespace"}}},{"name":"INSTANCE_IP","valueFrom":{"fieldRef":{"apiVersion":"v1","fieldPath":"status.podIP"}}},{"name":"SERVICE_ACCOUNT","valueFrom":{"fieldRef":{"apiVersion":"v1","fieldPath":"spec.serviceAccountName"}}},{"name":"HOST_IP","valueFrom":{"fieldRef":{"apiVersion":"v1","fieldPath":"status.hostIP"}}},{"name":"PROXY_CONFIG","value":"{}\n"},{"name":"ISTIO_META_POD_PORTS","value":"[\n    {\"name\":\"http\",\"containerPort\":8080,\"protocol\":\"TCP\"}\n    ,{\"name\":\"https\",\"containerPort\":8443,\"protocol\":\"TCP\"}\n]"},{"name":"ISTIO_META_APP_CONTAINERS","value":"wordpress"},{"name":"ISTIO_META_CLUSTER_ID","value":"Kubernetes"},{"name":"ISTIO_META_INTERCEPTION_MODE","value":"REDIRECT"},{"name":"ISTIO_META_WORKLOAD_NAME","value":"katsuoryuu-org-wordpress"},{"name":"ISTIO_META_OWNER","value":"kubernetes://apis/apps/v1/namespaces/wordpress/deployments/katsuoryuu-org-wordpress"},{"name":"ISTIO_META_MESH_ID","value":"cluster.local"},{"name":"TRUST_DOMAIN","value":"cluster.local"},{"name":"ISTIO_KUBE_APP_PROBERS","value":"{\"/app-health/wordpress/livez\":{\"httpGet\":{\"path\":\"/wp-admin/install.php\",\"port\":8080,\"scheme\":\"HTTP\"},\"timeoutSeconds\":5},\"/app-health/wordpress/readyz\":{\"httpGet\":{\"path\":\"/wp-login.php\",\"port\":8080,\"scheme\":\"HTTP\"},\"timeoutSeconds\":5}}"}],"image":"docker.io/istio/proxyv2:1.13.3","imagePullPolicy":"IfNotPresent","name":"istio-proxy","ports":[{"containerPort":15090,"name":"http-envoy-prom","protocol":"TCP"}],"readinessProbe":{"failureThreshold":30,"httpGet":{"path":"/healthz/ready","port":15021,"scheme":"HTTP"},"initialDelaySeconds":1,"periodSeconds":2,"successThreshold":1,"timeoutSeconds":3},"resources":{"limits":{"cpu":"2","memory":"1Gi"},"requests":{"cpu":"100m","memory":"128Mi"}},"securityContext":{"allowPrivilegeEscalation":false,"capabilities":{"drop":["ALL"]},"privileged":false,"readOnlyRootFilesystem":true,"runAsGroup":1337,"runAsNonRoot":true,"runAsUser":1337},"terminationMessagePath":"/dev/termination-log","terminationMessagePolicy":"File","volumeMounts":[{"mountPath":"/var/run/secrets/istio","name":"istiod-ca-cert"},{"mountPath":"/var/lib/istio/data","name":"istio-data"},{"mountPath":"/etc/istio/proxy","name":"istio-envoy"},{"mountPath":"/var/run/secrets/tokens","name":"istio-token"},{"mountPath":"/etc/istio/pod","name":"istio-podinfo"},{"mountPath":"/var/run/secrets/kubernetes.io/serviceaccount","name":"kube-api-access-r74bw","readOnly":true}]}],"dnsPolicy":"ClusterFirst","enableServiceLinks":true,"hostAliases":[{"hostnames":["status.localhost"],"ip":"127.0.0.1"}],"initContainers":[{"args":["istio-iptables","-p","15001","-z","15006","-u","1337","-m","REDIRECT","-i","*","-x","","-b","*","-d","15090,15021,15020"],"image":"docker.io/istio/proxyv2:1.13.3","imagePullPolicy":"IfNotPresent","name":"istio-init","resources":{"limits":{"cpu":"2","memory":"1Gi"},"requests":{"cpu":"100m","memory":"128Mi"}},"securityContext":{"allowPrivilegeEscalation":false,"capabilities":{"add":["NET_ADMIN","NET_RAW"],"drop":["ALL"]},"privileged":false,"readOnlyRootFilesystem":false,"runAsGroup":0,"runAsNonRoot":false,"runAsUser":0},"terminationMessagePath":"/dev/termination-log","terminationMessagePolicy":"File","volumeMounts":[{"mountPath":"/var/run/secrets/kubernetes.io/serviceaccount","name":"kube-api-access-r74bw","readOnly":true}]}],"nodeName":"nebula","preemptionPolicy":"PreemptLowerPriority","priority":0,"restartPolicy":"Always","schedulerName":"default-scheduler","securityContext":{"fsGroup":1337},"serviceAccount":"default","serviceAccountName":"default","terminationGracePeriodSeconds":30,"tolerations":[{"effect":"NoExecute","key":"node.kubernetes.io/not-ready","operator":"Exists","tolerationSeconds":300},{"effect":"NoExecute","key":"node.kubernetes.io/unreachable","operator":"Exists","tolerationSeconds":300}],"volumes":[{"emptyDir":{"medium":"Memory"},"name":"istio-envoy"},{"emptyDir":{},"name":"istio-data"},{"downwardAPI":{"defaultMode":420,"items":[{"fieldRef":{"apiVersion":"v1","fieldPath":"metadata.labels"},"path":"labels"},{"fieldRef":{"apiVersion":"v1","fieldPath":"metadata.annotations"},"path":"annotations"}]},"name":"istio-podinfo"},{"name":"istio-token","projected":{"defaultMode":420,"sources":[{"serviceAccountToken":{"audience":"istio-ca","expirationSeconds":43200,"path":"istio-token"}}]}},{"configMap":{"defaultMode":420,"name":"istio-ca-root-cert"},"name":"istiod-ca-cert"},{"name":"wordpress-data","persistentVolumeClaim":{"claimName":"katsuoryuu-org-wordpress"}},{"name":"kube-api-access-r74bw","projected":{"defaultMode":420,"sources":[{"serviceAccountToken":{"expirationSeconds":3607,"path":"token"}},{"configMap":{"items":[{"key":"ca.crt","path":"ca.crt"}],"name":"kube-root-ca.crt"}},{"downwardAPI":{"items":[{"fieldRef":{"apiVersion":"v1","fieldPath":"metadata.namespace"},"path":"namespace"}]}}]}}]},"status":{"conditions":[{"lastTransitionTime":"2022-06-07T11:42:21Z","status":"True","type":"Initialized"},{"lastTransitionTime":"2022-06-07T11:44:07Z","status":"True","type":"Ready"},{"lastTransitionTime":"2022-06-07T11:44:07Z","status":"True","type":"ContainersReady"},{"lastTransitionTime":"2022-06-07T11:38:55Z","status":"True","type":"PodScheduled"}],"containerStatuses":[{"containerID":"docker://8a0e02954d6333f519f48acaad5967d127183d28959ea66ad64af0571e40bbab","image":"istio/proxyv2:1.13.3","imageID":"docker-pullable://istio/proxyv2@sha256:e8986efce46a7e1fcaf837134f453ea2b5e0750a464d0f2405502f8ddf0e2cd2","lastState":{},"name":"istio-proxy","ready":true,"restartCount":0,"started":true,"state":{"running":{"startedAt":"2022-06-07T11:43:31Z"}}},{"containerID":"docker://ee4d668df3e4a6eaedbc7194ae7ef410e97e2833aa113b21e93e4c664ba2bb20","image":"bitnami/wordpress:5.9.2-debian-10-r4","imageID":"docker-pullable://bitnami/wordpress@sha256:609a48d5d1fbda160ffe045f70e77e7221c10eded249cff150d00bdd7d8c41c3","lastState":{},"name":"wordpress","ready":true,"restartCount":0,"started":true,"state":{"running":{"startedAt":"2022-06-07T11:43:31Z"}}}],"hostIP":"192.168.80.224","initContainerStatuses":[{"containerID":"docker://08a9a89d7e7883ac964a984b9f8b2ecbbe81a82ac540bc698b9b838a88ce8a04","image":"istio/proxyv2:1.13.3","imageID":"docker-pullable://istio/proxyv2@sha256:e8986efce46a7e1fcaf837134f453ea2b5e0750a464d0f2405502f8ddf0e2cd2","lastState":{},"name":"istio-init","ready":true,"restartCount":0,"state":{"terminated":{"containerID":"docker://08a9a89d7e7883ac964a984b9f8b2ecbbe81a82ac540bc698b9b838a88ce8a04","exitCode":0,"finishedAt":"2022-06-07T11:42:21Z","reason":"Completed","startedAt":"2022-06-07T11:42:21Z"}}}],"phase":"Running","podIP":"172.17.0.27","podIPs":[{"ip":"172.17.0.27"}],"qosClass":"Burstable","startTime":"2022-06-07T11:38:55Z"}}"##;

    #[derive(Serialize)]
    pub struct User {
        id: String,
        metadata: Metadata,
        is_active: bool,
        balance: String,
        age: i32,
        eye_color: String,
        name: String,
        gender: String,
        company: String,
        email: String,
        phone: String,
        friends: Vec<Friend>,
        favorite_fruit: String,
    }

    impl Default for User {
        fn default() -> Self {
            Self {
                id: "5973782bdb9a930533b05cb2".into(),
                metadata: Metadata::default(),
                is_active: true,
                balance: "$1,446.35".into(),
                age: 32,
                eye_color: "green".into(),
                name: "Logan Keller".into(),
                gender: "male".into(),
                company: "ARTIQ".into(),
                email: "logankeller@artiq.com".into(),
                phone: "+1 (952) 533-2258".into(),
                friends: Friends::default().into(),
                favorite_fruit: "banana".into(),
            }
        }
    }
    #[derive(Serialize)]
    pub struct Metadata {
        namespace: String,
        annotations: HashMap<String, String>,
    }

    impl Default for Metadata {
        fn default() -> Self {
            let mut map = HashMap::new();
            map.insert("mesh.controller.io/group".to_string(), "env-01".to_string());

            Self {
                namespace: "Randoms".to_string(),
                annotations: map,
            }
        }
    }

    #[derive(Serialize)]
    pub struct Friends(Vec<Friend>);

    #[derive(Serialize)]
    pub struct Friend {
        id: i32,
        name: String,
    }

    impl Default for Friends {
        fn default() -> Self {
            Friends(vec![
                Friend {
                    id: 0,
                    name: "Colon Salazar".into(),
                },
                Friend {
                    id: 1,
                    name: "French Mcneil".into(),
                },
                Friend {
                    id: 2,
                    name: "Carol Martin".into(),
                },
            ])
        }
    }

    impl From<Friends> for Vec<Friend> {
        fn from(f: Friends) -> Self {
            f.0
        }
    }

    #[test]
    fn test_com_type() {
        let large = ComType::from(100);
        let mid = ComType::from(50);
        let low = ComType::from(10);
        assert!(large == large);
        assert!(mid < large);
        assert!(low < mid);
    }

    #[test]
    fn test_proc_macro() {
        let lex: LinkedList<LexOperator> = precompile_lex!(.metadata[1,2,4-6,hello]);
        println!("{:?}", lex);
    }

    #[cfg(not(feature = "jq"))]
    #[test]
    fn test_query_precompile() {
        let lex = precompile_lex!(.friends[1].name);
        println!("{:?}", lex);
        let data = User::default();
        let query_res = query(data, lex);
        println!("{:?}", query_res.unwrap());
    }

    #[cfg(not(feature = "jq"))]
    #[test]
    fn test_query_multiple_results_precompile() {
        let lex = precompile_lex!(.friends[1,2].name);
        println!("{:?}", lex);
        let data = User::default();
        let query_res = query(data, lex);
        println!("{:?}", query_res.unwrap());
    }

    #[test]
    #[cfg(not(feature = "jq"))]
    fn test_query() {
        let lex = compile(".friends[1].name").unwrap();
        println!("{:?}", lex);
        let data = User::default();
        let query_res = query(data, lex);
        println!("{:?}", query_res.unwrap());
    }

    #[cfg(not(feature = "jq"))]
    #[test]
    fn test_query_multiple_results() {
        let lex = compile(".friends[1,2].name").unwrap();
        println!("{:?}", lex);
        let data = User::default();
        let query_res = query(data, lex);
        println!("{:?}", query_res.unwrap());
    }

    #[cfg(not(feature = "jq"))]
    #[test]
    fn test_query_results_subident() {
        let lex = compile(".metadata.namespace.").unwrap();
        println!("{:?}", lex);
        let data = User::default();
        let query_res = query(data, lex);
        println!("{:?}", query_res.unwrap());
    }

    #[cfg(feature = "jq")]
    #[test]
    fn jq_test() {
        let lexical = ".metadata.namespace";
        let value: Value = serde_json::from_str(TEST_OBJECT_RAW).unwrap();
        let res = query(value, lexical);
        println!("{:?}", res)
    }
}
