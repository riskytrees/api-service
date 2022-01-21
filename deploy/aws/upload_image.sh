aws ecr get-login-password --region us-east-2 | docker login --username AWS --password-stdin 571837724543.dkr.ecr.us-east-2.amazonaws.com

# Dev
docker tag riskyserv:latest 571837724543.dkr.ecr.us-east-2.amazonaws.com/riskyserv-dev:latest
docker push 571837724543.dkr.ecr.us-east-2.amazonaws.com/riskyserv-dev:latest
