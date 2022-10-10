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

def test_projects_get_empty():
    r = requests.get('http://localhost:8000/projects')

    res = r.json()

    assert(res['ok'] == True)
    assert("Got" in res['message'])
    assert(len(res['result']['projects']) == 0)


def test_project_post():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'})

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')


def test_projects_get():
    r = requests.get('http://localhost:8000/projects')

    res = r.json()

    assert(res['ok'] == True)
    assert("Got" in res['message'])
    assert(len(res['result']['projects']) > 0)

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
    assert(res['result']['id'] != '')


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


def test_project_tree_put_with_nodes():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'})

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'Have some Nodes'})

    res = r.json()
    tree_id = res['result']['id']

    # PUTing the tree list should return the modified version
    r = requests.put('http://localhost:8000/projects/' + str(project_id) + '/trees/' + str(tree_id), json = {
        'title': 'My Tree',
        'nodes': [{
            'id': "0",
            'title': "I'm the root",
            'description': "Hello",
            'modelAttributes': {
                'randomProp': {
                    'value_int': 150
                },
                'otherProp': {
                    'value_string': 'test'
                }
            },
            'conditionAttribute': 'config[\'test\'] == 150',
            'children': ["1", "2"],

        }, {
            'id': "1",
            'title': "I'm a child",
            'description': "Hello",
            'modelAttributes': {},
            'conditionAttribute': '',
            'children': [],

        }, {
            'id': "2",
            'title': "I'm the forgotten child",
            'description': "Hello",
            'modelAttributes': {},
            'conditionAttribute': '',
            'children': [],

        }],
        'rootNodeId': '0'
        })

    res = r.json()

    assert(res['ok'] == True)
    assert(res['result']['title'] == 'My Tree')
    assert(len(res['result']['nodes']) == 3)

    for node in res['result']['nodes']:
        if node['id'] == '0':
            assert(len(node['children']) == 2)
            assert(node['description'] == 'Hello')
            assert(node['modelAttributes']['randomProp']['value_int'] == 150)
            assert(node['modelAttributes']['otherProp']['value_string'] == 'test')
            assert(node['conditionAttribute'] == 'config[\'test\'] == 150')

def test_get_model_list():
    r = requests.get('http://localhost:8000/models')

    res = r.json()
    models = res['result']['models']

    assert(len(models) > 0)

def test_get_and_update_project_model():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'})

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.get('http://localhost:8000/projects/' + str(project_id) + '/model')

    assert(r.json()['result']['modelId'] == '')

    r = requests.put('http://localhost:8000/projects/' + str(project_id) + '/model', json = {'modelId': 'test'})
    assert(r.json()['ok'] == True)

    r = requests.get('http://localhost:8000/projects/' + str(project_id) + '/model')

    assert(r.json()['result']['modelId'] == 'test')

def test_get_node_response():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'})

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'Have some Nodes'})

    res = r.json()
    tree_id = res['result']['id']

    # PUTing the tree list should return the modified version
    r = requests.put('http://localhost:8000/projects/' + str(project_id) + '/trees/' + str(tree_id), json = {
        'title': 'My Tree',
        'nodes': [{
            'id': "unique-node-0",
            'title': "I'm the root",
            'description': "Hello",
            'modelAttributes': {
                'randomProp': {
                    'value_int': 150
                },
                'otherProp': {
                    'value_string': 'test'
                }
            },
            'conditionAttribute': 'config[\'test\'] == 150',
            'children': ["1", "2"],

        }, {
            'id': "1",
            'title': "I'm a child",
            'description': "Hello",
            'modelAttributes': {},
            'conditionAttribute': '',
            'children': [],

        }, {
            'id': "2",
            'title': "I'm the forgotten child",
            'description': "Hello",
            'modelAttributes': {},
            'conditionAttribute': '',
            'children': [],

        }],
        'rootNodeId': '0'
        })

    create_res = r.json()
    assert(create_res['ok'] == True)

    r = requests.get('http://localhost:8000/nodes/unique-node-0')
    node_res = r.json()
    print(r.json())
    assert(node_res['ok'] == True)


