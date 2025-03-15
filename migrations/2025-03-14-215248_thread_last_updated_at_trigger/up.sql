CREATE TRIGGER thread_last_updated_at_after_thread_update
    AFTER UPDATE ON threads
    FOR EACH ROW
BEGIN
    UPDATE threads
    SET last_updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.id;
END;

CREATE TRIGGER thread_last_updated_at_after_new_message
    AFTER INSERT ON messages
    FOR EACH ROW
BEGIN
    UPDATE threads
    SET last_updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.thread_id;
END;