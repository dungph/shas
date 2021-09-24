CREATE TABLE public.user_account (
    "username" text PRIMARY KEY,
    "password" text NOT NULL,
    "admin" bool NOT NULL DEFAULT false
);

CREATE TABLE public.entity (
    "public_key" bytea PRIMARY KEY,
    "entity_data" jsonb NOT NULL DEFAULT '{}'::jsonb,
    "manager" text NULL,
    CONSTRAINT check_data_is_object CHECK (jsonb_typeof(entity_data) = 'object'),
    CONSTRAINT manager_fk FOREIGN KEY ("manager") REFERENCES user_account("username") ON DELETE SET NULL
);

CREATE TABLE public.entity_log (
    "log_id" bigserial PRIMARY KEY,
    "public_key" bytea NOT NULL,
    "entity_data" jsonb NOT NULL,
    "log_timestamp" timestamptz(0) NOT NULL DEFAULT now(),
    CONSTRAINT log_fk FOREIGN KEY ("public_key") REFERENCES entity ("public_key") ON DELETE CASCADE
);

-- function from https://stackoverflow.com/a/36043269
-- Thank you Savinkov!
CREATE OR REPLACE FUNCTION jsonb_diff_val(val1 JSONB, val2 JSONB)
RETURNS JSONB AS $$
DECLARE
  result JSONB;
  v RECORD;
BEGIN
   result = val1;
   FOR v IN SELECT * FROM jsonb_each(val2) LOOP
     IF result @> jsonb_build_object(v.key,v.value)
        THEN result = result - v.key;
     ELSIF result ? v.key THEN CONTINUE;
     ELSE
        result = result || jsonb_build_object(v.key,'null');
     END IF;
   END LOOP;
   RETURN result;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION log_entity ()
RETURNS TRIGGER
language plpgsql
as $$
declare v record;
begin

    insert into entity_log ("public_key", "entity_data") 
    values (NEW.public_key, jsonb_diff_val(NEW.entity_data, OLD.entity_data))
    on conflict ("log_id") do nothing;
    return NEW;
end;$$;

--DROP TRIGGER IF EXISTS insert_logging on entity;
CREATE TRIGGER insert_logging AFTER INSERT OR UPDATE ON entity
FOR EACH ROW EXECUTE PROCEDURE log_entity();
