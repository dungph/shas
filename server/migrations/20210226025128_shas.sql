CREATE TABLE public.peer (
	"peer_id" bytea NOT NULL UNIQUE,
	"password" text NULL,
	"admin" bool NOT NULL DEFAULT false,
	CONSTRAINT peer_pk PRIMARY KEY ("peer_id")
);

CREATE TABLE public.peer_subscription (
	"subject" bytea NOT NULL,
	"object" bytea NOT NULL,
	"manage" bool NOT NULL DEFAULT false,
	"read" bool NOT NULL DEFAULT false,
	"request" bool NOT NULL DEFAULT false,
	CONSTRAINT peer_subscription_pk PRIMARY KEY ("subject", "object"),
	CONSTRAINT peer_subscription_subject_fk FOREIGN KEY ("subject") REFERENCES public.peer("peer_id") ON DELETE CASCADE,
	CONSTRAINT peer_subscription_object_fk FOREIGN KEY ("object") REFERENCES public.peer("peer_id") ON DELETE CASCADE
);


CREATE TABLE public.field (
	"peer_id" bytea NOT NULL,
	"field_name" text NOT NULL,
	"data" jsonb NOT NULL DEFAULT '{}'::jsonb,
	CONSTRAINT field_pk PRIMARY KEY ("peer_id", "field_name"),
	CONSTRAINT field_name_check CHECK (((field_name)::text ~ '^[a-zA-Z0-9_]*$'::text)),
	CONSTRAINT field_fk FOREIGN KEY ("peer_id") REFERENCES public.peer("peer_id") ON DELETE CASCADE,
	CONSTRAINT field_check_is_object CHECK (jsonb_typeof(data) = 'object'),
	CONSTRAINT field_alias_check CHECK (
		(data->'alias' IS NULL) OR
		(jsonb_typeof(data->'alias') = 'string')
	),
	CONSTRAINT field_disabled_check CHECK (
		(data->'disabled' IS NULL) OR
		(jsonb_typeof(data->'disabled') = 'boolean')
	),
	CONSTRAINT field_min_check CHECK (
		(data->'min' IS NULL) OR
		(jsonb_typeof(data->'min') = 'number')
	),
	CONSTRAINT field_max_check CHECK (
		(data->'max' IS NULL) OR
		(jsonb_typeof(data->'max') = 'number')
	),
	CONSTRAINT field_regex_check CHECK (
		(data->'regex' IS NULL) OR
		(jsonb_typeof(data->'regex') = 'string')
	),
	CONSTRAINT field_value_number_check CHECK (
		(jsonb_typeof(data->'value') = 'number' AND (
			(data->'min' IS NULL AND data->'max' IS NULL) OR
			(data->'min' IS NULL AND (data->'value')::numeric <= (data->'max')::numeric) OR
			((data->'min')::numeric <= (data->'value')::numeric AND data->'max' IS NULL) OR
			((data->'min')::numeric <= (data->'value')::numeric AND (data->'value')::numeric <= (data->'max')::numeric)
		)) 
		OR (jsonb_typeof(data->'value') != 'number')
	),

	CONSTRAINT field_value_text_length_check CHECK (
		(jsonb_typeof(data->'value') = 'string' AND (
			(data->'regex' IS NULL) OR ((data->'value')::text ~ (data->'regex')::text)
		)) 
		OR (jsonb_typeof(data->'value') != 'string')
	)
);

CREATE TABLE public.field_log (
	"peer_id" bytea NOT NULL,
	"field_name" text NOT NULL,
	"timest" timestamptz(0) NOT NULL DEFAULT now(),
	"data" jsonb NULL,
	CONSTRAINT field_log_pk PRIMARY KEY ("field_name", "peer_id", "timest"),
	CONSTRAINT field_log_fk FOREIGN KEY ("peer_id", "field_name") REFERENCES field("peer_id", "field_name") ON DELETE CASCADE
);

-- function from https://stackoverflow.com/a/36043269
-- Thank you Savinkov!
CREATE OR REPLACE FUNCTION jsonb_diff_val(val1 JSONB,val2 JSONB)
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

CREATE OR REPLACE FUNCTION log_field ()
RETURNS TRIGGER
language plpgsql
as $$
declare v record;
begin

	insert into field_log ("peer_id", "field_name", "data") 
	values (NEW.peer_id, NEW.field_name, jsonb_diff_val(NEW.data,OLD.data))
	on conflict ("peer_id", "field_name", "timest") do nothing;
	return NEW;
end;$$;

--DROP TRIGGER IF EXISTS field_insert_logging on field;
CREATE TRIGGER field_insert_logging AFTER INSERT OR UPDATE ON field
FOR EACH ROW EXECUTE PROCEDURE log_field();
