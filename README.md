# DemoAnalyserRS

Counter-Strike 1.6의 리플레이 파일(`.dem`)을 **분석**하기 위한 도구입니다.

> 목표: GoldSrc Demo 포맷을 읽어 프레임/메시지/이벤트 등 구조화된 데이터로 변환하고,
> 분석에 활용할 수 있게 합니다.

## 기능

- `.dem` 바이너리 파싱 (CS 1.6 replay)
- 플레이어 트레이싱 및 theta값 표시

![예시 아웃풋](./test/output.gif)

## 설치 / 빌드

### 요구사항

- Rust (stable 권장)
- Cargo

### 빌드

```bash
cargo build --release
```
