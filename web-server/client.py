import requests

def get(url):
    try:
        response = requests.get(url)
        if response.status_code != 200:
            print(f'Received error status code {response.status_code} from {url}')
        print(response.content.decode('utf-8'))
    except requests.exceptions.ConnectionError as e:
        print(f'Failed to connect to the server: {e}')

if __name__ == '__main__':
    get('http://localhost:7878')
    get('http://localhost:7878/health')
    get('http://localhost:7878/sleep')
