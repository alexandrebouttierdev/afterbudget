-- Date du dernier export réussi.
--
-- Ajoutée par la migration v3. L'ALTER est rejoué uniquement si la colonne
-- n'existe pas encore (garde dans core/db/migrations.rs) : ce SQL n'est donc
-- jamais exécuté deux fois sur une base déjà à jour.
ALTER TABLE app_settings ADD COLUMN last_export_date TEXT;
