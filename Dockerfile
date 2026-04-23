FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY frontend_flask.py .
COPY templates/ ./templates/
COPY app/static/ ./app/static/
EXPOSE 5001
ENV API_URL=http://backend:7001
CMD ["python", "frontend_flask.py"]