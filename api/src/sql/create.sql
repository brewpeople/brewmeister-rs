CREATE TABLE IF NOT EXISTS steps (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    title TEXT,
    description TEXT,
    target_temperature INTEGER,
    duration INTEGER,
    confirmation_required INTEGER
);

CREATE TABLE IF NOT EXISTS recipes (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    title TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS recipe_steps (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    recipe_id INTEGER NOT NULL,
    step_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    FOREIGN KEY(recipe_id) REFERENCES recipes(id)
    FOREIGN KEY(step_id) REFERENCES steps(id)
);

CREATE TABLE IF NOT EXISTS brews (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    title TEXT,
    description TEXT,
    recipe_id INTEGER NOT NULL,
    FOREIGN KEY(recipe_id) REFERENCES recipes(id)
);

CREATE TABLE IF NOT EXISTS brew_measurements (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    brew_id INTEGER,
    timestamp INTEGER,
    brew_temperature REAL,
    ambient_temperature REAL,
    heating INTEGER,
    error INTEGER,
    FOREIGN KEY(brew_id) REFERENCES brews(id)
);
