CREATE OR REPLACE FUNCTION check_max_players()
RETURNS TRIGGER
LANGUAGE plpgsql
AS
$$
DECLARE
    l_max_players INTEGER;
BEGIN
    SELECT max_players INTO l_max_players FROM games WHERE name = NEW.game;

    IF l_max_players IS NOT NULL AND NEW.ordinal > l_max_players THEN
        RAISE EXCEPTION 'TOO-LARGE-ORDINAL';
    END IF;

    RETURN NEW;
END;
$$;
