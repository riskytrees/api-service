import requests

def test_auth_login():
  r = requests.post('http://localhost:8000/auth/login', json = {'email':'test@example.com'})

  res = r.json()

  assert(res['ok'] == True)
  assert("created" in res['message'])

  # Do it again, this time should return same user
  r = requests.post('http://localhost:8000/auth/login', json = {'email':'test@example.com'})

  res = r.json()

  assert(res['ok'] == True)
  assert("created" not in res['message'])


def test_project_post():
  r = requests.post('http://localhost:8000/projects', json = {'title':'test project'})

  res = r.json()

  assert(res['ok'] == True)
  assert("created" in res['message'])
  assert(res['result']['title'] == 'test project')
