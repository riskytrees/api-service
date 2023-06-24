import uuid
import requests
import os

# The following is a Test JWT created by using the following, non-prod JWT secret:
#   testjwttestjwttestjwttestjwttestjwt
TEST_JWT = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbWFpbCI6InRlc3RAZXhhbXBsZS5jb20ifQ.nnzKa34M7aloJO94_OQyJIEBCnr2tKshriSb0lNNd9A"
TEST_HEADERS = {
    'Authorization': TEST_JWT
}

def test_auth_login():
    r = requests.post('http://localhost:8000/auth/login', json = {'email':'test@example.com'})

    res = r.json()

    assert(res['ok'] == True)
    assert("Got request URI" in res['message'])

    # Should include a loginRequest
    assert("mocklab.io" in res['result']['loginRequest'])


def test_projects_get_empty():
    r = requests.get('http://localhost:8000/projects', headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("Got" in res['message'])
    assert(len(res['result']['projects']) == 0)


def test_project_post():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

def test_project_put():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'Test Tree Put'}, headers = TEST_HEADERS)
    res = r.json()
    assert(res['ok'] == True)

    r = requests.get('http://localhost:8000/projects/' + project_id + '/trees', headers = TEST_HEADERS)
    res = r.json()
    assert(res['ok'] == True)
    assert(len(res['result']['trees']) == 1)

    r = requests.put('http://localhost:8000/projects/' + project_id, json = {'title':'other project'}, headers = TEST_HEADERS)
    res = r.json()
    print(res)
    assert(res['ok'] == True)
    assert("updated" in res['message'])
    assert(res['result']['title'] == 'other project')

    r = requests.get('http://localhost:8000/projects/' + project_id + '/trees', headers = TEST_HEADERS)
    res = r.json()
    assert(res['ok'] == True)
    assert(len(res['result']['trees']) == 1)

    

def test_projects_get():
    r = requests.get('http://localhost:8000/projects', headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("Got" in res['message'])
    assert(len(res['result']['projects']) > 0)

def test_project_tree_post():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'bad things'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("Added tree" in res['message'])
    assert(res['result']['title'] == 'bad things')
    assert(res['result']['id'] != '')


def test_project_trees_get():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'bad things'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("Added tree" in res['message'])
    assert(res['result']['title'] == 'bad things')

    # GETing the tree list should return a single tree.
    r = requests.get('http://localhost:8000/projects/' + str(project_id) + '/trees', headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert(res['result']['trees'][0]['title'] == 'bad things')


def test_project_tree_get():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'bad things'}, headers = TEST_HEADERS)

    res = r.json()
    tree_id = res['result']['id']

    # GETing the tree list should return a single tree.
    r = requests.get('http://localhost:8000/projects/' + str(project_id) + '/trees/' + str(tree_id), headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert(res['result']['title'] == 'bad things')


def test_project_tree_put():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'Test Tree Put'}, headers = TEST_HEADERS)

    res = r.json()
    tree_id = res['result']['id']

    # PUTing the tree list should return the modified version
    r = requests.put('http://localhost:8000/projects/' + str(project_id) + '/trees/' + str(tree_id), json = {
        'title': 'Test Confirm Tree Put',
        'nodes': [],
        'rootNodeId': ''
        }, headers = TEST_HEADERS)

    res = r.json()
    print(res)
    assert(res['ok'] == True)
    assert(res['result']['title'] == 'Test Confirm Tree Put')


def test_project_tree_put_with_nodes():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'Have some Nodes'}, headers = TEST_HEADERS)

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
        }, headers = TEST_HEADERS)

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
    r = requests.get('http://localhost:8000/models', headers = TEST_HEADERS)

    res = r.json()
    models = res['result']['models']

    assert(len(models) > 0)

def test_get_and_update_project_model():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.get('http://localhost:8000/projects/' + str(project_id) + '/model', headers = TEST_HEADERS)

    assert(r.json()['result']['modelId'] == '')

    r = requests.put('http://localhost:8000/projects/' + str(project_id) + '/model', json = {'modelId': 'test'}, headers = TEST_HEADERS)
    assert(r.json()['ok'] == True)

    r = requests.get('http://localhost:8000/projects/' + str(project_id) + '/model', headers = TEST_HEADERS)

    assert(r.json()['result']['modelId'] == 'test')

def test_get_node_response():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'Have some Nodes'}, headers = TEST_HEADERS)

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
        }, headers = TEST_HEADERS)

    create_res = r.json()
    assert(create_res['ok'] == True)

    r = requests.get('http://localhost:8000/nodes/unique-node-0', headers = TEST_HEADERS)
    node_res = r.json()
    print(r.json())
    assert(node_res['ok'] == True)


def test_tree_with_subtree():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'Have some Nodes'}, headers = TEST_HEADERS)

    res = r.json()
    tree_id = res['result']['id']

    unique_id = uuid.uuid4().urn

    # PUTing the tree list should return the modified version
    r = requests.put('http://localhost:8000/projects/' + str(project_id) + '/trees/' + str(tree_id), json = {
        'title': 'My Tree',
        'nodes': [{
            'id': unique_id,
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
        'rootNodeId': unique_id
        }, headers = TEST_HEADERS)

    create_res = r.json()
    assert(create_res['ok'] == True)

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'With subtree'}, headers = TEST_HEADERS)
    res = r.json()
    tree_id = res['result']['id']

    r = requests.put('http://localhost:8000/projects/' + str(project_id) + '/trees/' + str(tree_id), json = {
        'title': 'My Tree',
        'nodes': [{
            'id': "root-id",
            'title': "I'm the root",
            'description': "Hello",
            'modelAttributes': {

            },
            'conditionAttribute': '',
            'children': [unique_id],

        }],
        'rootNodeId': 'root-id'
        }, headers = TEST_HEADERS)

    create_res = r.json()
    assert(create_res['ok'] == True)

    r = requests.get('http://localhost:8000/projects/' + str(project_id) + '/trees/' + str(tree_id) + "/dag/down", headers = TEST_HEADERS)
    dag_res = r.json()
    assert(create_res['ok'] == True)

def test_get_configs():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.get('http://localhost:8000/projects/' + project_id + "/configs", headers = TEST_HEADERS)

    res = r.json()
    assert(res['ok'] == True)
    assert("Got" in res['message'])
    assert(len(res['result']['ids']) == 0)

def test_create_config():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    r = requests.post('http://localhost:8000/projects/' + project_id + "/configs", json = {
      "attributes": {
        "Hello": "Test"
      }  
    }, headers = TEST_HEADERS)
    res = r.json()
    assert(res['ok'] == True)

    r = requests.get('http://localhost:8000/projects/' + project_id + "/configs", headers = TEST_HEADERS)

    res = r.json()
    assert(res['ok'] == True)
    assert("Got" in res['message'])
    assert(len(res['result']['ids']) == 1)

def test_create_config():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    # Selected config should error (beacuse none is selected)
    r = requests.get('http://localhost:8000/projects/' + project_id + "/config", headers = TEST_HEADERS)
    res = r.json()
    assert(res['ok'] == False)


    r = requests.post('http://localhost:8000/projects/' + project_id + "/configs", json = {
      "attributes": {
        "Hello": "Test"
      }  
    }, headers = TEST_HEADERS)
    res = r.json()
    assert(res['ok'] == True)

    config_id = res['result']['id']

    r = requests.put('http://localhost:8000/projects/' + project_id + "/config", json = {
      "desiredConfig": config_id
    }, headers = TEST_HEADERS)
    res = r.json()
    print(res)
    assert(res['ok'] == True)

    # Selected config should not error (beacuse config_id is selected)
    r = requests.get('http://localhost:8000/projects/' + project_id + "/config", headers = TEST_HEADERS)
    res = r.json()
    assert(res['ok'] == True)



def test_update_config():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    # Create config
    r = requests.post('http://localhost:8000/projects/' + project_id + "/configs", json = {
      "attributes": {
        "Hello": "Test"
      }  
    }, headers = TEST_HEADERS)
    res = r.json()
    assert(res['ok'] == True)

    config_id = res['result']['id']

    # Update config
    r = requests.put('http://localhost:8000/projects/' + project_id + "/configs/" + config_id, json = {
      "attributes": {
        "New": "Value"
      }  
    }, headers = TEST_HEADERS)
    res = r.json()
    assert(res['ok'] == True)

    # Check that it was updated
    r = requests.get('http://localhost:8000/projects/' + project_id + "/configs/" + config_id, headers = TEST_HEADERS)
    res = r.json()
    assert(res['ok'] == True)
    assert(res['result']['attributes']['New'] == 'Value')

def test_condition_resolution():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']

    # Create config
    r = requests.post('http://localhost:8000/projects/' + project_id + "/configs", json = {
      "attributes": {
        "test": "150",
        "other": True
      }  
    }, headers = TEST_HEADERS)
    res = r.json()
    assert(res['ok'] == True)

    config_id = res['result']['id']

    r = requests.put('http://localhost:8000/projects/' + project_id + "/config", json = {
      "desiredConfig": config_id
    }, headers = TEST_HEADERS)
    res = r.json()
    print(res)
    assert(res['ok'] == True)

    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'Have some Nodes'}, headers = TEST_HEADERS)

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
            'conditionAttribute': '150 == 150',
            'children': ["1", "2", "3", "4"],

        }, {
            'id': "1",
            'title': "I'm a child",
            'description': "Hello",
            'modelAttributes': {},
            'conditionAttribute': 'config[\'test\'] == "150"',
            'children': [],

        }, {
            'id': "2",
            'title': "I'm the forgotten child",
            'description': "Hello",
            'modelAttributes': {},
            'conditionAttribute': '',
            'children': [],

        }, {
            'id': "3",
            'title': "Third",
            'description': "Hello",
            'modelAttributes': {
                'randomProp': {
                    'value_int': 150
                },
                'otherProp': {
                    'value_string': 'test'
                }
            },
            'conditionAttribute': '125 == 150',
            'children': ["1", "2"],

        }, {
            'id': "4",
            'title': "Four",
            'description': "Hello",
            'modelAttributes': {
                'randomProp': {
                    'value_int': 150
                },
                'otherProp': {
                    'value_string': 'test'
                }
            },
            'conditionAttribute': 'config[\'otttther\'] == true',
            'children': [],

        }],
        'rootNodeId': '0'
        }, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert(res['result']['title'] == 'My Tree')
    assert(len(res['result']['nodes']) == 5)
    condition_results = [res['result']['nodes'][0]['conditionResolved'],
                         res['result']['nodes'][1]['conditionResolved'],
                         res['result']['nodes'][2]['conditionResolved'],
                         res['result']['nodes'][3]['conditionResolved'],
                         res['result']['nodes'][4]['conditionResolved']
                        ]

    assert(list(filter(lambda r : r == True, condition_results)).count(True) == 3)


def test_condition_no_config():
    r = requests.post('http://localhost:8000/projects', json = {'title':'test project'}, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert("created" in res['message'])
    assert(res['result']['title'] == 'test project')

    project_id = res['result']['id']


    r = requests.post('http://localhost:8000/projects/' + str(project_id) + '/trees', json = {'title':'Have some Nodes'}, headers = TEST_HEADERS)

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
            'conditionAttribute': '150 == 150',
            'children': ["1", "2", "3"],

        }, {
            'id': "1",
            'title': "I'm a child",
            'description': "Hello",
            'modelAttributes': {},
            'conditionAttribute': 'config[\'test\'] == "150"',
            'children': [],

        }, {
            'id': "2",
            'title': "I'm the forgotten child",
            'description': "Hello",
            'modelAttributes': {},
            'conditionAttribute': '',
            'children': [],

        }, {
            'id': "3",
            'title': "Third",
            'description': "Hello",
            'modelAttributes': {
                'randomProp': {
                    'value_int': 150
                },
                'otherProp': {
                    'value_string': 'test'
                }
            },
            'conditionAttribute': '125 == 150',
            'children': ["1", "2"],

        }],
        'rootNodeId': '0'
        }, headers = TEST_HEADERS)

    res = r.json()

    assert(res['ok'] == True)
    assert(res['result']['title'] == 'My Tree')
    assert(len(res['result']['nodes']) == 4)
    condition_results = [res['result']['nodes'][0]['conditionResolved'],
                         res['result']['nodes'][1]['conditionResolved'],
                         res['result']['nodes'][2]['conditionResolved'],
                         res['result']['nodes'][3]['conditionResolved']
                        ]

    assert(list(filter(lambda r : r == True, condition_results)).count(True) == 0)
