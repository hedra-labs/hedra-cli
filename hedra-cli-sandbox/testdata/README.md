# Test TLS material

The fake broker (`src/fake_broker.rs`) serves TLS with `server.pem` and
`server.key`, a certificate for `localhost` and `127.0.0.1` signed by `ca.pem`.
Tests trust it with `HEDRA_CLI_CA_BUNDLE=<this directory>/ca.pem`. These are
test-only keys; nothing else trusts this CA. The CA's own key is not kept.

Regenerate all three together (valid for 100 years):

```sh
openssl ecparam -name prime256v1 -genkey -noout | openssl pkcs8 -topk8 -nocrypt -out ca.key
openssl req -x509 -new -key ca.key -sha256 -days 36500 -subj "/CN=hedra-cli fake broker test CA" \
  -addext "basicConstraints=critical,CA:TRUE" -addext "keyUsage=critical,keyCertSign,cRLSign" -out ca.pem
openssl ecparam -name prime256v1 -genkey -noout | openssl pkcs8 -topk8 -nocrypt -out server.key
openssl req -new -key server.key -subj "/CN=localhost" -out server.csr
printf 'basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature\nextendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost,IP:127.0.0.1\n' > server.ext
openssl x509 -req -in server.csr -CA ca.pem -CAkey ca.key -CAcreateserial -days 36500 -sha256 -extfile server.ext -out server.pem
rm -f server.csr server.ext ca.srl ca.key
```
