#!/bin/bash

# Create certificate directories
mkdir -p etc/ssl/certs etc/ssl/private

# Generate private key
openssl genpkey -algorithm RSA -out etc/ssl/private/server.key -pkcs8

# Generate certificate signing request
openssl req -new -key etc/ssl/private/server.key -out etc/ssl/server.csr \
    -subj "/CN=localhost/O=Development/C=US"

# Create certificate extensions file for SAN
cat > etc/ssl/server.ext << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
DNS.2 = 127.0.0.1
IP.1 = 127.0.0.1
EOF

# Generate self-signed certificate
openssl x509 -req -in etc/ssl/server.csr -signkey etc/ssl/private/server.key \
    -out etc/ssl/certs/server.crt -days 365 -extensions v3_req -extfile etc/ssl/server.ext

# Clean up
rm etc/ssl/server.csr etc/ssl/server.ext

# Set proper permissions
chmod 600 etc/ssl/private/server.key
chmod 644 etc/ssl/certs/server.crt

echo "✅ Self-signed certificate generated!"
echo "Certificate: etc/ssl/certs/server.crt"
echo "Private key: etc/ssl/private/server.key"
echo ""
echo "To trust this certificate in your browser:"
echo "1. Visit https://localhost:8443"
echo "2. Click 'Advanced' -> 'Proceed to localhost (unsafe)'"
echo "3. Or add the certificate to your system's trust store"