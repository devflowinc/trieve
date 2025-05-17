CREATE TABLE IF NOT EXISTS experiments (
    id UUID,
    name String,
    t1_name String,
    t1_split Float32,
    control_name String,
    control_split Float32,
    dataset_id UUID,
    created_at DateTime DEFAULT now(),
    updated_at DateTime DEFAULT now(),
)
ORDER BY (created_at, id)
PARTITION BY
    (dataset_id);

CREATE TABLE IF NOT EXISTS experiment_user_assignments (
    id UUID,
    experiment_id UUID,
    user_id String,
    dataset_id UUID,
    treatment_name String,
    created_at DateTime DEFAULT now(),
    updated_at DateTime DEFAULT now(),
)
ORDER BY (created_at, id)
PARTITION BY
    (experiment_id);
    