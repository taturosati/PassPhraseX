FROM nginx:alpine

COPY ./nginx.conf /etc/nginx/nginx.conf
COPY ./startup.sh /usr/local/bin/startup.sh

RUN apk add certbot

RUN chmod +x /usr/local/bin/startup.sh

CMD ["/usr/local/bin/startup.sh"]