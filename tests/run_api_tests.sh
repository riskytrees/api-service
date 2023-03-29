
./tests/simulatedb.sh

cd tests/
python3 -m venv env;
source env/bin/activate && python3 -m pip install -r requirements.txt;
source env/bin/activate && source mocklab_odic.env && pytest

docker rm $(docker stop $(docker ps -a -q --filter ancestor=mongo --format="{{.ID}}"))
