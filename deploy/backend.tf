provider aws {}

terraform {
  backend "s3" {
    bucket = "riskytrees-tfstate"
    key = "us-east-2/main/dev/api-service.tfstate"
    region = "us-east-2"    
  }
}
