provider aws {
  region = "us-east-2"
}

// Deploy DocumentDB (mongodb-like) via RDS
resource "aws_docdb_cluster" "docdb" {
  cluster_identifier      = "riskytrees-dev"
  engine                  = "docdb"
  master_username         = "clusteradmin"
  master_password         = "${var.database_cluster_password}"
  backup_retention_period = 5
  preferred_backup_window = "07:00-09:00"
  skip_final_snapshot     = true
}

resource "aws_docdb_cluster_instance" "cluster_instances" {
  count              = 1
  identifier         = "riskytrees-dev-${count.index}"
  cluster_identifier = aws_docdb_cluster.docdb.id
  instance_class     = "db.t3.medium"
}

// Upload image to ECR

// Deploy ECR via ECS
