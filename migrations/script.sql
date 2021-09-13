CREATE TABLE IF NOT EXISTS teloxide_dialogues(
    chat_id BIGINT PRIMARY KEY,
    dialogue BYTEA NOT NULL
);

CREATE TABLE IF NOT EXISTS groups(
    id SERIAL PRIMARY KEY,
    name VARCHAR (255)
);

CREATE TABLE IF NOT EXISTS lessons_groups(
    lesson INTEGER REFERENCES schedule(id),
    "group" INTEGER REFERENCES groups(id),
    PRIMARY KEY(lesson, "group")
);

CREATE TABLE IF NOT EXISTS schedule(
    id SERIAL PRIMARY KEY,
    subject INTEGER REFERENCES subjects(id),
    type lesson_types,
    teacher INTEGER REFERENCES teachers(id),
    day_of_week  days_of_week,
    time TIME,
    distribution distribution_week,
    info VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS students(
    id SERIAL PRIMARY KEY,
    telegram_id INTEGER UNIQUE,
    "group" INTEGER REFERENCES groups(id),
    last_name VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS students_subjects(
    student INTEGER REFERENCES students(id),
    subject INTEGER REFERENCES subjects(id),
    PRIMARY KEY(student, subject)
);

CREATE TABLE IF NOT EXISTS subjects(
    id SERIAL PRIMARY KEY,
    name VARCHAR(255),
    choice BOOLEAN,
    info VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS teachers(
    id SERIAL PRIMARY KEY,
    last_name VARCHAR(255),
    first_name VARCHAR(255),
    patronymic_name VARCHAR(255),
    telegram VARCHAR(255),
    email VARCHAR(255),
    phone_number VARCHAR(12),
    UNIQUE(last_name, first_name, patronymic_name)
);

CREATE TYPE days_of_week AS ENUM ('monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday', 'sunday');

CREATE TYPE distribution_week AS ENUM ('first', 'second', 'all');

CREATE TYPE lesson AS(
    subject_name VARCHAR(255),
    lesson_type lesson_types,
    time TIME,
    teacher_name TEXT,
    info VARCHAR(255)
);

CREATE TYPE lesson_types AS ENUM ('lecture', 'practice', 'laboratory work');


CREATE FUNCTION get_current_schedule(user_id INTEGER) RETURNS SETOF lesson
    LANGUAGE SQL
AS
$$
SELECT * FROM get_today_schedule(user_id)
WHERE time <= (CURRENT_TIME AT TIME ZONE 'EETDST')::time + INTERVAL '1 hours 55 minutes' AND time + INTERVAL '1 hours 35 minutes' >= (CURRENT_TIME AT TIME ZONE 'EET')::time
LIMIT 2
$$;

CREATE FUNCTION get_distribution_week() RETURNS distribution_week
    LANGUAGE SQL
AS
$$
SELECT
    CASE (CURRENT_DATE - '2021-02-01') / 7 % 2
        WHEN 0 THEN 'first'::distribution_week
        ELSE 'second'::distribution_week
        END
$$;

CREATE FUNCTION get_schedule(day days_of_week, user_id INTEGER)
    RETURNS TABLE(subject_name CHARACTER VARYING, lesson_type lesson_types, "time" TIME WITHOUT TIME ZONE, teacher_name TEXT, info CHARACTER VARYING)
    LANGUAGE plpgsql
AS
$$
DECLARE
    student_val students%ROWTYPE;
BEGIN
    SELECT * INTO student_val FROM students WHERE telegram_id = user_id;
    RETURN QUERY SELECT DISTINCT
                     subjects.name,
                     schedule.type,
                     schedule.time,
                     concat_ws(' ', teachers.last_name,  teachers.first_name, teachers.patronymic_name),
                     schedule.info
                 FROM schedule
                          JOIN subjects ON subjects.id = schedule.subject
                          LEFT JOIN teachers ON teachers.id = schedule.teacher
                 WHERE
                     (distribution = 'all'::distribution_week OR distribution = (SELECT * FROM get_distribution_week())) AND
                         day_of_week = day AND EXISTS(SELECT * FROM lessons_groups WHERE "group" = student_val."group" AND lesson = schedule.id) AND
                     (NOT subjects.choice OR EXISTS(SELECT * FROM students_subjects WHERE student = student_val.id AND subject = schedule.subject))
                 ORDER BY schedule.time;
    RETURN;
END
$$;

CREATE FUNCTION get_today_schedule(user_id INTEGER) RETURNS SETOF lesson
    LANGUAGE SQL
AS
$$
SELECT * FROM get_schedule(trim(to_char(now(), 'day'))::days_of_week, user_id)
$$;

CREATE FUNCTION get_week_schedule(user_id INTEGER, week distribution_week)
    RETURNS TABLE(day_of_week days_of_week, subject_name CHARACTER VARYING, lesson_type lesson_types, "time" TIME WITHOUT TIME ZONE, teacher_name TEXT, info CHARACTER VARYING)
    LANGUAGE plpgsql
as
$$
DECLARE
    student_val students%ROWTYPE;
BEGIN
    SELECT * INTO student_val FROM students WHERE telegram_id = user_id;
    RETURN QUERY SELECT DISTINCT
                     schedule.day_of_week,
                     subjects.name,
                     schedule.type,
                     schedule.time,
                     concat_ws(' ', teachers.last_name,  teachers.first_name, teachers.patronymic_name),
                     schedule.info
                 FROM schedule
                          JOIN subjects ON subjects.id = schedule.subject
                          LEFT JOIN teachers ON teachers.id = schedule.teacher
                 WHERE
                     (distribution = 'all'::distribution_week OR distribution = week) AND
                     EXISTS(SELECT * FROM lessons_groups WHERE "group" = student_val."group" AND lesson = schedule.id) AND
                     (NOT subjects.choice OR EXISTS(SELECT * FROM students_subjects WHERE student = student_val.id AND subject = schedule.subject))
                 ORDER BY schedule.day_of_week, schedule.time;
    RETURN;
END
$$;
