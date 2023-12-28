#!/bin/sh

# Check if certificate does not exist
if [ ! -f "/etc/letsencrypt/live/${DOMAIN}/fullchain.pem" ]; then
    # Get certificate
    certbot certonly \
        --standalone \
        --agree-tos \
        --no-eff-email \
        --non-interactive \
        --email="${EMAIL}" \
        -d "${DOMAIN}"
fi


# Start nginx
nginx -g "daemon off;"

# keep container running & renew certificate
while :; do
    certbot renew --standalone --non-interactive
    sleep 12h
done

