"""Tạo cấu hình riêng cho giai đoạn 1: mặc định relay xác thực, giữ baseline đã đo.

Không tự chọn địa chỉ hạ tầng, map, item, sức chứa hoặc quyền account. Đầu vào
phải là profile DB/managed-player-round đã có; việc đóng gói fixture thuộc M8.
Cấu hình có thông tin hạ tầng chỉ được ghi mới bên trong local/ với mode 0600.
"""
import argparse
import copy
import ipaddress
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def endpoint(value):
    address, separator, port = value.rpartition(':')
    try:
        ip = ipaddress.IPv4Address(address)
        number = int(port)
    except (ValueError, ipaddress.AddressValueError) as error:
        raise argparse.ArgumentTypeError('Cần IPv4:port cụ thể') from error
    if not separator or not 1 <= number <= 65535 or ip.is_unspecified or ip.is_multicast or ip == ipaddress.IPv4Address('255.255.255.255'):
        raise argparse.ArgumentTypeError('Cần endpoint unicast có cổng khác 0')
    return str(ip)+':'+str(number)


def phase1(config, relay_bind, relay_advertise):
    result = copy.deepcopy(config)
    two = result.get('diagnostic_two_lobby', {})
    if (two.get('account_admission', {}).get('profile_mode') != 'database_zero'
            or not two.get('managed_rooms', {}).get('player_round')
            or not two.get('managed_rooms', {}).get('player_choices')
            or not isinstance(two.get('fixed_room'), dict)):
        raise ValueError('Cần baseline account DB và managed player round đã đo')
    relay = copy.deepcopy(result.get('authenticated_relay'))
    if not isinstance(relay, dict):
        raise ValueError('Cần budgets relay đã chọn trong baseline; tool không tự cấp tải')
    relay.update(bind=endpoint(relay_bind), advertise=endpoint(relay_advertise))
    result['authenticated_relay'] = relay
    result.pop('udp_relay', None)
    two['fixed_room']['endpoint_policy'] = 'authenticated_relay'
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', type=Path, required=True)
    parser.add_argument('--relay-bind', type=endpoint, required=True)
    parser.add_argument('--relay-advertise', type=endpoint, required=True)
    parser.add_argument('--database', type=Path, required=True)
    parser.add_argument('--launch-socket', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    local = (ROOT/'local').resolve()
    if not out.is_relative_to(local) or out == local:
        parser.error('Thông tin hạ tầng chỉ ghi vào local/; không ghi file tracked')
    if not out.parent.is_dir():
        parser.error('Thư mục đầu ra riêng phải tồn tại')
    try:
        value = phase1(json.loads(args.base.read_text()), args.relay_bind, args.relay_advertise)
        value['diagnostic_two_lobby']['account_admission'].update(
            database=str(args.database.resolve()), launch_socket=str(args.launch_socket.resolve()))
    except (ValueError, OSError) as error:
        parser.error(str(error))
    # Exclusive create: preserve existing operator inputs and symlinks. Do not
    # print private endpoints/configuration or copy credentials to stdout.
    fd = os.open(out, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'w') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')
    print('Đã tạo cấu hình riêng: relay xác thực. Native scope theo PLAN; budgets cần kiểm M8.')


if __name__ == '__main__':
    main()
