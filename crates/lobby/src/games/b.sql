DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'check_ordinal') THEN
        CREATE TRIGGER check_ordinal
        BEFORE INSERT OR UPDATE ON players
        FOR EACH ROW
        EXECUTE FUNCTION check_max_players();
    END IF;
END $$;
