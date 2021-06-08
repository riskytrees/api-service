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
