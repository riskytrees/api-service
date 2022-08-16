cd ..
docker-compose up -d

cd tests/
python3 -m venv env;
source env/bin/activate && python3 -m pip install -r requirements.txt;
source env/bin/activate && pytest

cd ..
docker-compose down