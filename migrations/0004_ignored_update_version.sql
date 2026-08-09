-- Version signalée comme ignorée par l'utilisateur dans la bannière de mise
-- à jour. Tant qu'elle est la plus récente connue, elle n'est plus proposée.
--
-- Ajoutée par la migration v4. L'ALTER est rejoué uniquement si la colonne
-- n'existe pas encore (garde dans core/db/migrations.rs).
ALTER TABLE app_settings ADD COLUMN ignored_update_version TEXT;
