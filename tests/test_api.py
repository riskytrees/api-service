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


def test_project_tree_post():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'})

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'bad things'})

    res = r.json()

    assert(res['ok'] == True)
    assert("Added tree" in res['message'])
    assert(res['result']['title'] == 'bad things')


def test_project_trees_get():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'})

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'bad things'})

    res = r.json()

    assert(res['ok'] == True)
    assert("Added tree" in res['message'])
    assert(res['result']['title'] == 'bad things')

    # GETing the tree list should return a single tree.
    r = requests.get('http://localhost:8000/projects/' + str(project_id) + '/trees')

    res = r.json()
    print(res)

    assert(res['ok'] == True)
    assert(res['result']['trees'][0]['title'] == 'bad things')


def test_project_tree_get():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'})

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'bad things'})

    res = r.json()
    tree_id = res['result']['id']

    # GETing the tree list should return a single tree.
    r = requests.get('http://localhost:8000/projects/' + str(project_id) + '/trees/' + str(tree_id))

    res = r.json()
    print(res)

    assert(res['ok'] == True)
    assert(res['result']['title'] == 'bad things')


def test_project_tree_put():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'})

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'Test Tree Put'})

    res = r.json()
    tree_id = res['result']['id']

    # PUTing the tree list should return the modified version
    r = requests.put('http://localhost:8000/projects/' + str(project_id) + '/trees/' + str(tree_id), json = {
        'title': 'Test Confirm Tree Put',
        'nodes': [],
        'rootNodeId': ''
        })

    res = r.json()

    assert(res['ok'] == True)
    assert(res['result']['title'] == 'Test Confirm Tree Put')
