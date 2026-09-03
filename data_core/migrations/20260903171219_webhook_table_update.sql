ALTER TABLE analytics.webhooks
    DROP CONSTRAINT webhooks_name_key;

ALTER TABLE analytics.webhooks
    ADD CONSTRAINT webhooks_url_key UNIQUE (url);
