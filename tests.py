import json
import requests
import unittest

import pre_tests


class TestTmt(unittest.TestCase):
    url: str = 'http://localhost:8080'

    def _authorize(self):
        data = {'client_id': 'sam.raker+1@gmail.com',
                'client_secret': 'password1'}
        resp = requests.post(f'{self.url}/authorize', data=json.dumps(data),
                             headers={'Content-Type': 'application/json'})
        return resp.json()

    def _logout(self, token: str):
        resp = requests.post(
            f'{self.url}/logout',
            headers={'Authorization': f'Bearer {token}'})
        return resp.json()


class TestTmtAuth(TestTmt):
    def test_authorize(self):
        data = {'client_id': 'sam.raker+1@gmail.com',
                'client_secret': 'password1'}
        resp = requests.post(f'{self.url}/authorize', data=json.dumps(data),
                             headers={'Content-Type': 'application/json'})
        self.assertEqual(resp.status_code, 200)
        jsn = resp.json()
        self.assertTrue('access_token' in jsn)
        self.assertTrue('token_type' in jsn)

    def test_authorize_idempotent(self):
        resp1 = self._authorize()['access_token']
        resp2 = self._authorize()['access_token']
        self.assertEqual(resp1, resp2)

    def test_private(self):
        token = self._authorize()['access_token']
        resp = requests.get(f'{self.url}/private',
                            headers={'Authorization': f'Bearer {token}'})
        self.assertEqual(resp.status_code, 200)

    def test_logout(self):
        token = self._authorize()['access_token']
        logged_in_resp = requests.get(
            f'{self.url}/private',
            headers={'Authorization': f'Bearer {token}'})
        self.assertEqual(logged_in_resp.status_code, 200)
        logout_resp = requests.post(
            f'{self.url}/logout',
            headers={'Authorization': f'Bearer {token}'})
        self.assertEqual(logout_resp.status_code, 200)
        jsn = logout_resp.json()
        self.assertTrue('session_id' in jsn)
        self.assertTrue(jsn['ok'])
        logged_out_resp = requests.get(
            f'{self.url}/private',
            headers={'Authorization': f'Bearer {token}'})
        self.assertEqual(logged_out_resp.status_code, 400)


class TestTmtApi(TestTmt):
    token: str = None

    def _logout(self):
        if self.token:
            super()._logout(self.token)
            self.token = None

    def _login(self):
        resp = self._authorize()
        self.token = resp['access_token']

    def tearDown(self):
        self._logout()


if __name__ == '__main__':
    pre_tests.main()
    input('Is tmt-web running?')
    unittest.main()
