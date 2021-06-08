docker kill $(docker ps -q);
docker rm $(docker ps -a -q)
./simulatedb.sh

python3 -m venv env;
source env/bin/activate && python3 -m pip install -r requirements.txt;
source env/bin/activate && pytest
